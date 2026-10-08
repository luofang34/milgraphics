//! Definitions, projections and output checks shared by the robustness tests.

use milgraphics::render::{
    FixedAdvanceMetrics, GeoShape, LocalEquirectangular, Projection, ScreenPoint, ScreenShape,
};
use milgraphics::{
    Budget, Config, Construction, ControlPoint, GeoPoint, GraphicDefinition, GraphicId, RenderPlan,
    SymbolId, View, construct, render,
};

pub(crate) const PHASE_LINE: &str = "140300";
pub(crate) const NAMED_AREA: &str = "120200";
pub(crate) const MAIN_ATTACK: &str = "151403";
pub(crate) const AIR_CORRIDOR: &str = "170100";
pub(crate) const RANGE_FAN: &str = "242200";
pub(crate) const BYPASS_EASY: &str = "270601";

/// `VV0325S0` + entity + `0000`: `version` 11 or 15, `status` 0 or 1.
pub(crate) fn symbol(version: u8, status: u8, entity: &str) -> SymbolId {
    SymbolId::parse(&format!("{version:02}0325{status}000{entity}0000")).unwrap()
}

pub(crate) fn control_point(lon: f64, lat: f64) -> ControlPoint {
    ControlPoint::ground(GeoPoint::new(lon, lat).unwrap())
}

pub(crate) fn definition(entity: &str, points: &[(f64, f64)]) -> GraphicDefinition {
    GraphicDefinition::new(
        GraphicId::new("g").unwrap(),
        symbol(11, 0, entity),
        points
            .iter()
            .map(|&(lon, lat)| control_point(lon, lat))
            .collect(),
    )
}

/// Orthographic globe centred on (`lon0`, `lat0`); the far hemisphere is
/// invisible.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Globe {
    pub(crate) lon0: f64,
    pub(crate) lat0: f64,
    pub(crate) radius_px: f64,
}

impl Projection for Globe {
    fn project(&self, p: GeoPoint) -> Option<ScreenPoint> {
        let (phi, lam) = (p.lat().to_radians(), (p.lon() - self.lon0).to_radians());
        let phi0 = self.lat0.to_radians();
        let cos_c = phi0.sin() * phi.sin() + phi0.cos() * phi.cos() * lam.cos();
        (cos_c > 0.0).then(|| ScreenPoint {
            x: self.radius_px * phi.cos() * lam.sin(),
            y: -self.radius_px * (phi0.cos() * phi.sin() - phi0.sin() * phi.cos() * lam.cos()),
        })
    }

    fn unproject(&self, _s: ScreenPoint) -> Option<GeoPoint> {
        None
    }

    fn segment_may_be_visible(&self, a: GeoPoint, b: GeoPoint) -> bool {
        // Every point of an arc of angular length θ lies within θ/2 of an
        // end, so ends more than 90° + θ/2 from the view centre hide it all.
        let unit = |p: GeoPoint| {
            let (phi, lam) = (p.lat().to_radians(), p.lon().to_radians());
            [phi.cos() * lam.cos(), phi.cos() * lam.sin(), phi.sin()]
        };
        let dot = |u: [f64; 3], v: [f64; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
        let (ua, ub) = (unit(a), unit(b));
        let centre = unit(GeoPoint::new(self.lon0, self.lat0).unwrap());
        let half = dot(ua, ub).clamp(-1.0, 1.0).acos() / 2.0;
        let limit = -half.sin() - 1e-9;
        !(dot(ua, centre) < limit && dot(ub, centre) < limit)
    }
}

pub(crate) fn view() -> View {
    View::new(0, 0)
}

/// A construction result that can be compared: errors by their message.
pub(crate) fn built(def: &GraphicDefinition, config: &Config) -> Result<Construction, String> {
    construct(def, config).map_err(|e| e.to_string())
}

/// A plan result that can be compared: errors by their message.
pub(crate) fn rendered(
    c: &Construction,
    projection: &dyn Projection,
    budget: &Budget,
) -> Result<RenderPlan, String> {
    render(
        c,
        &view(),
        projection,
        &FixedAdvanceMetrics::default(),
        budget,
    )
    .map_err(|e| e.to_string())
}

fn finite(v: f64, what: &str) {
    assert!(v.is_finite(), "{what} is not finite: {v}");
}

fn screen_point(p: ScreenPoint, what: &str) {
    finite(p.x, what);
    finite(p.y, what);
}

fn lon_lat(p: [f64; 2], what: &str) {
    assert!(
        (-180.0..=180.0).contains(&p[0]) && (-90.0..=90.0).contains(&p[1]),
        "{what} out of range: {p:?}"
    );
}

/// Every output coordinate is finite and geographic ones are in range.
pub(crate) fn assert_plan_sane(plan: &RenderPlan) {
    for item in &plan.geo {
        let (GeoShape::Lines(parts) | GeoShape::Polygons(parts)) = &item.shape;
        parts
            .iter()
            .flatten()
            .for_each(|&p| lon_lat(p, "geo vertex"));
    }
    for item in &plan.screen {
        let (ScreenShape::Polyline(p) | ScreenShape::Polygon(p)) = &item.shape;
        p.iter().for_each(|&p| screen_point(p, "screen vertex"));
    }
    for l in &plan.labels {
        lon_lat([l.anchor.lon(), l.anchor.lat()], "label anchor");
        l.screen
            .iter()
            .for_each(|&p| screen_point(p, "label screen"));
        l.corners
            .iter()
            .flatten()
            .for_each(|&p| screen_point(p, "label corner"));
        finite(l.rotation_deg, "label rotation");
        finite(l.width_px, "label width");
        l.offset_em.iter().for_each(|&v| finite(v, "label offset"));
    }
    for h in &plan.handles {
        lon_lat([h.at.lon(), h.at.lat()], "handle");
        h.screen
            .iter()
            .for_each(|&p| screen_point(p, "handle screen"));
    }
}

/// Vertices in the plan, geographic tier then screen tier.
pub(crate) fn vertex_counts(plan: &RenderPlan) -> (usize, usize) {
    let geo = plan
        .geo
        .iter()
        .map(|i| match &i.shape {
            GeoShape::Lines(p) | GeoShape::Polygons(p) => p.iter().map(Vec::len).sum::<usize>(),
        })
        .sum();
    let screen = plan
        .screen
        .iter()
        .map(|i| match &i.shape {
            ScreenShape::Polyline(p) | ScreenShape::Polygon(p) => p.len(),
        })
        .sum();
    (geo, screen)
}

/// The projections every definition is rendered under: a globe centred on
/// the first control point (so the graphic is visible), one centred on the
/// opposite side, and a local plane at `scale`.
pub(crate) fn projections(def: &GraphicDefinition, scale: f64) -> Vec<Box<dyn Projection>> {
    let (lon, lat) = def
        .points
        .first()
        .map_or((0.0, 0.0), |p| (p.position.lon(), p.position.lat()));
    let globe = |lon0: f64, lat0: f64| -> Box<dyn Projection> {
        Box::new(Globe {
            lon0,
            lat0,
            radius_px: 400.0,
        })
    };
    vec![
        globe(lon, lat),
        globe(lon + 180.0, -lat),
        Box::new(LocalEquirectangular::new(lon, lat, scale, 96.0)),
    ]
}

/// Constructs and renders `def` under every projection, asserting sane
/// output. Returns the construction result for further checks.
pub(crate) fn run_everywhere(
    def: &GraphicDefinition,
    config: &Config,
    scale: f64,
) -> Result<Construction, String> {
    let c = built(def, config)?;
    for p in projections(def, scale) {
        if let Ok(plan) = rendered(&c, p.as_ref(), &config.budget) {
            assert_plan_sane(&plan);
        }
    }
    Ok(c)
}
