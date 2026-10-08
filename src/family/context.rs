//! State shared by family constructors while building one construction.

use crate::budget::{BudgetError, VertexMeter};
use crate::construction::{
    Construction, Decoration, EmbeddedSymbol, GeoGeometry, GeoPart, HandleSpec, LabelSpec, PartId,
    PartRole, ScreenDecoration,
};
use crate::definition::GraphicDefinition;
use crate::family::{Config, ConstructError};
use crate::geo::GeoPoint;
use crate::geodesy::Earth;
use crate::style::{Fill, Palette, Stroke};
use crate::support::SymbolSpec;

/// Builder for one construction: geodesics, budgets, and the parts so far.
#[derive(Debug)]
pub(crate) struct Ctx<'a> {
    pub(crate) earth: &'a Earth,
    pub(crate) config: &'a Config,
    pub(crate) palette: Palette,
    pub(crate) spec: &'static SymbolSpec,
    meter: VertexMeter,
    parts: Vec<GeoPart>,
    decorations: Vec<ScreenDecoration>,
    labels: Vec<LabelSpec>,
    handles: Vec<HandleSpec>,
    symbols: Vec<EmbeddedSymbol>,
}

impl<'a> Ctx<'a> {
    pub(crate) fn new(
        earth: &'a Earth,
        config: &'a Config,
        palette: Palette,
        spec: &'static SymbolSpec,
    ) -> Self {
        Self {
            earth,
            config,
            palette,
            spec,
            meter: VertexMeter::new(&config.budget),
            parts: Vec::new(),
            decorations: Vec::new(),
            labels: Vec::new(),
            handles: Vec::new(),
            symbols: Vec::new(),
        }
    }

    /// Adds a single-point symbol the graphic embeds.
    pub(crate) fn add_symbol(&mut self, symbol: EmbeddedSymbol) {
        self.symbols.push(symbol);
    }

    /// The id the next part will get.
    pub(crate) fn next_part(&self) -> PartId {
        PartId(self.parts.len() as u16)
    }

    /// Geodesic polyline through `points`, densified; the last point is kept.
    pub(crate) fn geodesic_path(
        &mut self,
        points: &[GeoPoint],
    ) -> Result<Vec<GeoPoint>, ConstructError> {
        let mut out = Vec::new();
        for pair in points.windows(2) {
            if let [a, b] = pair {
                let inv = self.earth.inverse(*a, *b);
                let steps =
                    crate::geodesy::segment_count(inv.distance_m, self.config.geodesic_step_m);
                self.meter.take(steps)?;
                self.earth
                    .densify(*a, *b, self.config.geodesic_step_m, &mut out);
            }
        }
        if let Some(last) = points.last() {
            self.meter.take(1)?;
            out.push(*last);
        }
        Ok(out)
    }

    /// Geodesic closed ring through `points`, densified, without repeating
    /// the first point.
    pub(crate) fn geodesic_ring(
        &mut self,
        points: &[GeoPoint],
    ) -> Result<Vec<GeoPoint>, ConstructError> {
        let mut closed = points.to_vec();
        if let Some(first) = points.first() {
            closed.push(*first);
        }
        let mut ring = self.geodesic_path(&closed)?;
        ring.pop();
        Ok(ring)
    }

    /// Reserves vertices for geometry built outside the geodesic helpers.
    pub(crate) fn take_vertices(&mut self, count: usize) -> Result<(), BudgetError> {
        self.meter.take(count)
    }

    pub(crate) fn add_part(
        &mut self,
        role: PartRole,
        geometry: GeoGeometry,
        stroke: Option<Stroke>,
        fill: Fill,
    ) -> PartId {
        let id = self.next_part();
        self.parts.push(GeoPart {
            id,
            role,
            geometry,
            stroke,
            fill,
        });
        id
    }

    pub(crate) fn add_decoration(&mut self, decoration: Decoration) {
        self.decorations.push(ScreenDecoration(decoration));
    }

    pub(crate) fn add_label(&mut self, label: LabelSpec) {
        self.labels.push(label);
    }

    pub(crate) fn add_handles(&mut self, handles: impl IntoIterator<Item = HandleSpec>) {
        self.handles.extend(handles);
    }

    pub(crate) fn finish(
        self,
        definition: &GraphicDefinition,
        renderer_version: &'static str,
    ) -> Result<Construction, ConstructError> {
        let limit = self.config.budget.max_labels;
        if self.labels.len() > limit {
            let count = self.labels.len();
            return Err(BudgetError::Labels { count, limit }.into());
        }
        Ok(Construction {
            renderer_version,
            definition: definition.id.clone(),
            revision: definition.revision,
            parts: self.parts,
            decorations: self.decorations,
            labels: self.labels,
            handles: self.handles,
            symbols: self.symbols,
        })
    }
}
