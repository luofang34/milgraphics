import armyc2.c5isr.renderer.utilities.MilStdSymbol;
import armyc2.c5isr.renderer.utilities.RendererSettings;
import armyc2.c5isr.renderer.utilities.ShapeInfo;
import armyc2.c5isr.web.render.MultiPointHandler;
import armyc2.c5isr.web.render.WebRenderer;

import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.Font;
import java.awt.FontMetrics;
import java.awt.Graphics2D;
import java.awt.GraphicsEnvironment;
import java.awt.geom.Point2D;
import java.awt.image.BufferedImage;
import java.io.File;
import java.io.FileDescriptor;
import java.io.FileOutputStream;
import java.io.PrintStream;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.Map;

/**
 * Renders one tactical-graphic case with the pinned mil-sym-java and prints a
 * canonical JSON record on stdout.
 *
 * Arguments: font file, then one tab-separated case line:
 * id, symbol ID, control points ("lon,lat lon,lat ..."), scale, bbox, then
 * any number of KEY=VALUE modifiers (keys are mil-sym Modifiers constants).
 *
 * mil-sym keeps renderer settings in static state, so the caller runs one
 * JVM per case and this class pins every setting it relies on.
 */
public final class Oracle {
    static final String PROBE = "PL ALPHA 0123456789";
    /** Large case sets drop the GeoJSON output and round geographic coordinates to 1e-7 degrees (about 1 cm). */
    static final boolean COMPACT = Boolean.getBoolean("oracle.compact");

    public static void main(String[] args) throws Exception {
        // Upstream logs through stdout; keep stdout for the record alone.
        PrintStream record = new PrintStream(
                new FileOutputStream(FileDescriptor.out), true, "UTF-8");
        System.setOut(System.err);
        Font base = Font.createFont(Font.TRUETYPE_FONT, new File(args[0]));
        GraphicsEnvironment.getLocalGraphicsEnvironment().registerFont(base);
        String family = base.getFamily();
        RendererSettings settings = RendererSettings.getInstance();
        settings.setDeviceDPI(96);
        settings.setTextBackgroundMethod(RendererSettings.TextBackgroundMethod_NONE);
        settings.setLabelFont(family, Font.BOLD, 12);
        settings.setMPLabelFont(family, Font.BOLD, 12);

        String[] f = args[1].split("\t", -1);
        String id = f[0], symbol = f[1], points = f[2], bbox = f[4];
        double scale = Double.parseDouble(f[3]);
        Map<String, String> modifiers = new LinkedHashMap<>();
        for (int i = 5; i < f.length; i++) {
            int eq = f[i].indexOf('=');
            if (eq > 0) modifiers.put(f[i].substring(0, eq), f[i].substring(eq + 1));
        }
        int count = points.trim().isEmpty() ? 0 : points.trim().split("\\s+").length;

        StringBuilder out = new StringBuilder("{");
        field(out, "case", str(id)).append(',');
        field(out, "symbol", str(symbol)).append(',');
        field(out, "control_points", str(points)).append(',');
        field(out, "scale", num(scale)).append(',');
        field(out, "bbox", str(bbox)).append(',');
        field(out, "modifiers", map(modifiers)).append(',');
        field(out, "font_probe", probe(family)).append(',');
        String can = MultiPointHandler.canRenderMultiPoint(symbol, new LinkedHashMap<>(modifiers), count);
        field(out, "can_render", str(can)).append(',');
        MilStdSymbol mss = WebRenderer.RenderMultiPointAsMilStdSymbol(id, "", "", symbol, points, "",
                scale, bbox, new LinkedHashMap<>(modifiers), new LinkedHashMap<>());
        field(out, "symbol_shapes", shapes(mss == null ? null : mss.getSymbolShapes())).append(',');
        field(out, "modifier_shapes", shapes(mss == null ? null : mss.getModifierShapes()));
        String geojson = COMPACT ? null : WebRenderer.RenderSymbol(id, "", "", symbol, points, "clampToGround", scale, bbox,
                new LinkedHashMap<>(modifiers), new LinkedHashMap<>(), WebRenderer.OUTPUT_FORMAT_GEOJSON);
        if (!COMPACT) field(out.append(','), "geojson", str(geojson));
        record.println(out.append('}'));
    }

    /** Font family and advance of a fixed string, so metric drift between platforms is visible. */
    static String probe(String family) {
        Graphics2D g = new BufferedImage(8, 8, BufferedImage.TYPE_INT_ARGB).createGraphics();
        g.setFont(new Font(family, Font.BOLD, 12));
        FontMetrics m = g.getFontMetrics();
        return "{\"family\":" + str(g.getFont().getFamily()) + ",\"text\":" + str(PROBE)
                + ",\"width\":" + m.stringWidth(PROBE) + ",\"ascent\":" + m.getAscent()
                + ",\"descent\":" + m.getDescent() + "}";
    }

    static String shapes(ArrayList<ShapeInfo> list) {
        if (list == null) return "null";
        StringBuilder b = new StringBuilder("[");
        for (int i = 0; i < list.size(); i++) {
            if (i > 0) b.append(',');
            ShapeInfo s = list.get(i);
            b.append('{');
            field(b, "shape_type", Integer.toString(s.getShapeType())).append(',');
            field(b, "line_color", color(s.getLineColor())).append(',');
            field(b, "fill_color", color(s.getFillColor())).append(',');
            BasicStroke st = s.getStroke();
            field(b, "stroke_width", st == null ? "null" : num(st.getLineWidth())).append(',');
            field(b, "dash", st == null || st.getDashArray() == null ? "null" : floats(st.getDashArray())).append(',');
            field(b, "polylines", polylines(s.getPolylines())).append(',');
            field(b, "text", str(s.getModifierString())).append(',');
            Point2D p = s.getModifierPosition();
            field(b, "position", p == null ? "null" : "[" + coord(p.getX()) + "," + coord(p.getY()) + "]").append(',');
            field(b, "angle", num(s.getModifierAngle())).append(',');
            field(b, "justify", Integer.toString(s.getTextJustify()));
            b.append('}');
        }
        return b.append(']').toString();
    }

    static String polylines(ArrayList<ArrayList<Point2D>> lines) {
        if (lines == null) return "null";
        StringBuilder b = new StringBuilder("[");
        for (int i = 0; i < lines.size(); i++) {
            if (i > 0) b.append(',');
            b.append('[');
            ArrayList<Point2D> line = lines.get(i);
            for (int j = 0; j < line.size(); j++) {
                if (j > 0) b.append(',');
                b.append('[').append(coord(line.get(j).getX())).append(',').append(coord(line.get(j).getY())).append(']');
            }
            b.append(']');
        }
        return b.append(']').toString();
    }

    static String floats(float[] a) {
        StringBuilder b = new StringBuilder("[");
        for (int i = 0; i < a.length; i++) b.append(i > 0 ? "," : "").append(num(a[i]));
        return b.append(']').toString();
    }

    static String color(Color c) {
        return c == null ? "null" : str(String.format("#%02x%02x%02x%02x", c.getRed(), c.getGreen(), c.getBlue(), c.getAlpha()));
    }

    static String map(Map<String, String> m) {
        StringBuilder b = new StringBuilder("{");
        for (Map.Entry<String, String> e : m.entrySet()) {
            if (b.length() > 1) b.append(',');
            field(b, e.getKey(), str(e.getValue()));
        }
        return b.append('}').toString();
    }

    /** Shortest round-trip decimal; non-finite values become null so the record stays valid JSON. */
    static String num(double v) {
        if (Double.isNaN(v) || Double.isInfinite(v)) return "null";
        if (v == Math.rint(v) && Math.abs(v) < 1e15) return Long.toString((long) v);
        return Double.toString(v);
    }

    static String coord(double v) {
        return num(COMPACT && Double.isFinite(v) ? Math.round(v * 1e7) / 1e7 : v);
    }

    static StringBuilder field(StringBuilder b, String key, String json) {
        return b.append(str(key)).append(':').append(json);
    }

    static String str(String s) {
        if (s == null) return "null";
        StringBuilder b = new StringBuilder("\"");
        for (char c : s.toCharArray()) {
            switch (c) {
                case '"': b.append("\\\""); break;
                case '\\': b.append("\\\\"); break;
                case '\n': b.append("\\n"); break;
                case '\r': b.append("\\r"); break;
                case '\t': b.append("\\t"); break;
                default:
                    if (c < 0x20) b.append(String.format("\\u%04x", (int) c));
                    else b.append(c);
            }
        }
        return b.append('"').toString();
    }
}
