import armyc2.c5isr.renderer.utilities.DrawRules;
import armyc2.c5isr.renderer.utilities.MSInfo;
import armyc2.c5isr.renderer.utilities.MSLookup;
import armyc2.c5isr.renderer.utilities.SymbolUtilities;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.TreeMap;
import java.util.TreeSet;

/**
 * Prints one oracle case per multipoint graphic that the pinned mil-sym-java
 * knows for APP-6(D) (version 10), 2525D change 1 (11), 2525E change 1 (15)
 * and APP-6(E) change 2 (16), in
 * symbol sets 25, 45 and 46.
 *
 * Control points come from one canonical layout per draw rule. The layouts
 * follow the anchor-point semantics documented in DrawRules.java and
 * MODrawRules.java, in metres east/north of lon 20, lat 50, so that every
 * graphic is legible at 1:50,000 and cases of one rule are comparable.
 */
public final class Cases {
    static final int[] VERSIONS = {10, 11, 15, 16};
    static final int[] SETS = {25, 45, 46};
    static final double LON0 = 20.0, LAT0 = 50.0;
    static final double M_PER_DEG_LAT = 111132.0;
    static final double M_PER_DEG_LON = 111320.0 * Math.cos(Math.toRadians(LAT0));

    static final Map<String, double[][]> LAYOUTS = new LinkedHashMap<>();
    /** AM/AN/X values that make a rule's documented geometry valid; applied even if not listed by MSInfo. */
    static final Map<String, String[]> SIZE_MODS = new LinkedHashMap<>();
    static final Map<String, String> TEXT = new LinkedHashMap<>();
    /**
     * Layouts for "version:entity" where the standard's text gives other
     * control points than upstream's draw rule (xtask/src/specs/rules.rs,
     * standard_points): Withdraw and Withdraw Under Pressure take three in
     * 2525E and APP-6(E); Trip Wire, Bearing Line and Linear Target two.
     */
    static final Map<String, String> STANDARD_LAYOUT = Map.of(
            "15:342400", "cm:Line24", "16:342400", "cm:Line24",
            "15:342500", "cm:Line24", "16:342500", "cm:Line24",
            "11:290500", "cm:Line14", "16:290500", "cm:Line14",
            "16:220100", "cm:Line14", "16:240701", "cm:Line14");

    static double[][] p(double... v) {
        double[][] r = new double[v.length / 2][2];
        for (int i = 0; i < r.length; i++) { r[i][0] = v[2 * i]; r[i][1] = v[2 * i + 1]; }
        return r;
    }

    static void layout(String rules, double[][] pts) {
        for (String r : rules.split(",")) LAYOUTS.put(r, pts);
    }

    static {
        double[][] open4 = p(-3000, -500, -1000, 800, 1000, -200, 3000, 900);
        double[][] box = p(-2500, -1500, 2500, -1500, 2500, 1500, -2500, 1500);
        double[][] twoLine = p(-2500, -500, 2500, 500);
        // Open polylines: 4 points with gentle turns.
        layout("cm:Line1,cm:Line2,cm:Line7,cm:Line8,cm:Line13,cm:Line21,mo:Line1,mo:Line2,mo:Line3,mo:Line4,mo:Line5,mo:Line6,mo:Line7,mo:Line8", open4);
        layout("cm:Corridor1", p(-3000, -500, 0, 1000, 3000, 0));
        // Areas bounded by anchor points.
        layout("cm:Area1,cm:Area2,cm:Area3,cm:Area4,cm:Area9,cm:Area10,cm:Area20,cm:Area23,mo:Area1,mo:Area2", box);
        // Inner zone (points 1..N/2), then outer zone.
        layout("cm:Area26", p(-1000, -500, 1000, -500, 0, 1000, -2500, -1500, 2500, -1500, 0, 2500));
        // Arrow-shaped missions: vertex first, then the two arrow tips.
        layout("cm:Line3,cm:Area21", p(0, -1500, -3000, 1500, 3000, 1500));
        layout("cm:Line4", p(-2500, 0, 2500, 0));
        layout("cm:Line5", p(-2500, -500, 2500, 800));
        layout("cm:Line6", p(-3000, -1500, 0, 1500, 3000, -1500));
        // Arrow tip first, rear second.
        layout("cm:Line9,cm:Line20,cm:Line25,cm:Line28", p(2500, 500, -2500, -500));
        layout("cm:Line10", p(2500, 0, -2500, 0, 0, 1500));
        // Two parallel sides of a gap or crossing site.
        layout("cm:Line11,cm:Line16", p(-1500, -1500, -1500, 1500, 1500, -1500, 1500, 1500));
        // Centreline plus a point giving the width.
        layout("cm:Line12,cm:Line17,cm:Polyline1", p(-2500, 0, 2500, 0, 0, 1000));
        layout("cm:Line14,cm:Line18,cm:Line19", p(-2500, -500, 2500, 500));
        // Opening endpoints, then rear.
        layout("cm:Line22,cm:Line23,cm:Point12,cm:Area11,cm:Area12,cm:Area17,cm:Area24,cm:Area25", p(0, 2000, 0, -2000, -3500, 0));
        layout("cm:Line24,cm:Area27", p(-2500, 0, 1500, 0, 2000, 1500));
        layout("cm:Line33", p(-2500, 0, 1500, 0, 2000, 1500));
        layout("cm:Line29,cm:Area7", p(2500, 0, -1000, 2000, -1000, -2000));
        layout("cm:Line30", p(2500, 0, -2500, 0, -2500, 800));
        layout("cm:Line31", p(-3000, 0, 1000, 0, 1500, 1200, 2000, 2000));
        layout("cm:Line32", p(-3000, 0, 0, 1500, 3000, 0));
        // Escort: points 2 and 3 make the line, point 1 slides it.
        layout("cm:Line50", p(0, 1000, -2500, 0, 2500, 0));
        layout("cm:Line26", p(-3000, 2500, 0, 500, 0, -500, -3000, -2500));
        layout("cm:Line27", p(0, 0, 1200, 0, 2500, 1500, 5000, 1500));
        layout("cm:Area5", p(0, 1500, 0, -1500, 4000, 0));
        layout("cm:Area6,cm:Area15,cm:Area16,cm:Area19", p(0, 0, 2000, 0));
        layout("cm:Area8", p(0, 1500, 0, -1500, 4000, 2500, 4000, -2500));
        layout("cm:Area13", p(-2000, 0, 2000, 0));
        layout("cm:Area14", p(0, 0, 1500, 0, 3000, 0));
        // Points 1 and 2 the first arrow (tip, end), 3 and 4 the second (tip at
        // the end of the curve from point 2, end).
        layout("cm:Area18", p(0, 2500, 3500, 2500, 3500, -2500, 0, -2500));
        // Axes of advance: tip, intermediate, rear, then the width point.
        layout("cm:Axis1,cm:Axis2", p(3000, 1000, 500, 0, -3000, -500, 2000, 1250));
        // Single anchor point plus AM/AN size modifiers.
        layout("cm:Arc1,cm:Circular1,cm:Circular2,cm:Ellipse1,cm:Point17,cm:Point18,cm:Rectangular2", p(0, 0));
        layout("cm:Rectangular1,cm:Rectangular3", twoLine);
        // Wind barb: plot circle, then the shaft.
        layout("mo:Point5", p(0, 0, 0, 1500));

        SIZE_MODS.put("cm:Arc1", new String[] {"AM_DISTANCE=1000,3000", "AN_AZIMUTH=30,90"});
        SIZE_MODS.put("cm:Circular1", new String[] {"AM_DISTANCE=2000"});
        SIZE_MODS.put("cm:Circular2", new String[] {"AM_DISTANCE=1000,2000"});
        SIZE_MODS.put("cm:Ellipse1", new String[] {"AM_DISTANCE=3000,1500", "AN_AZIMUTH=30"});
        SIZE_MODS.put("cm:Point17", new String[] {"AM_DISTANCE=3000,2000", "AN_AZIMUTH=30"});
        SIZE_MODS.put("cm:Point18", new String[] {"AM_DISTANCE=1000,3000", "AN_AZIMUTH=45,90"});
        SIZE_MODS.put("cm:Rectangular1", new String[] {"AM_DISTANCE=2000"});
        SIZE_MODS.put("cm:Rectangular2", new String[] {"AM_DISTANCE=3000,2000", "AN_AZIMUTH=800"});
        SIZE_MODS.put("cm:Rectangular3", new String[] {"AM_DISTANCE=2000"});
        SIZE_MODS.put("cm:Corridor1", new String[] {"AM_DISTANCE=2000", "X_ALTITUDE_DEPTH=1000,3000"});

        TEXT.put("T_UNIQUE_DESIGNATION_1", "T1X");
        TEXT.put("T1_UNIQUE_DESIGNATION_2", "T2Y");
        TEXT.put("T2_UNIQUE_DESIGNATION_3", "T3Z");
        TEXT.put("H_ADDITIONAL_INFO_1", "HH");
        TEXT.put("H1_ADDITIONAL_INFO_2", "H1H");
        TEXT.put("H2_ADDITIONAL_INFO_3", "H2H");
        TEXT.put("W_DTG_1", "W0800Z");
        TEXT.put("W1_DTG_2", "W1200Z");
        TEXT.put("N_HOSTILE", "ENY");
        TEXT.put("C_QUANTITY", "5");
        TEXT.put("V_EQUIP_TYPE", "VV");
        TEXT.put("Y_LOCATION", "YY");
        TEXT.put("AS_COUNTRY", "US");
        TEXT.put("AP_TARGET_NUMBER", "AP1");
        TEXT.put("AP1_TARGET_NUMBER_EXTENSION", "X");
        TEXT.put("B_ECHELON", "XX");
        TEXT.put("Q_DIRECTION_OF_MOVEMENT", "45");
        TEXT.put("AM_DISTANCE", "2000");
        TEXT.put("AN_AZIMUTH", "45");
        TEXT.put("X_ALTITUDE_DEPTH", "1000,3000");
    }

    record Entry(int set, String basic, int version, MSInfo info) {}

    public static void main(String[] args) {
        MSLookup lookup = MSLookup.getInstance();
        TreeMap<String, Entry[]> byKey = new TreeMap<>();
        for (int vi = 0; vi < VERSIONS.length; vi++) {
            int v = VERSIONS[vi];
            for (String id : lookup.getIDList(v)) {
                MSInfo info = lookup.getMSLInfo(id, v);
                if (info == null || info.getDrawRule() == DrawRules.DONOTDRAW) continue;
                int set = info.getSymbolSet();
                if (!inSets(set)) continue;
                String basic = info.getBasicSymbolID();
                if (!SymbolUtilities.isMultiPoint(sidc(v, set, 0, entityOf(basic)))) continue;
                byKey.computeIfAbsent(basic, k -> new Entry[VERSIONS.length])[vi] = new Entry(set, basic, v, info);
            }
        }
        System.out.println("# Generated by: tools/oracle/oracle.sh --cases > tools/oracle/cases/all.tsv");
        System.out.println("# Every multipoint graphic of mil-sym-java in APP-6(D) (-app6d), 2525D ch1 (-d), 2525E ch1 (-e) and APP-6(E) ch2 (-app6e), symbol sets 25, 45, 46.");
        System.out.println("# Layouts follow the anchor-point semantics in DrawRules.java/MODrawRules.java; see tools/oracle/java/Cases.java.");
        System.out.println("# id\tsymbol ID\tcontrol points (lon,lat ...)\tscale\tbbox (W,S,E,N)\tKEY=VALUE modifiers...");
        List<String> extras = new ArrayList<>();
        TreeSet<String> seenRules = new TreeSet<>();
        // Extra cases use the first symbol of each (version, draw rule).
        for (Map.Entry<String, Entry[]> e : byKey.entrySet()) {
            for (Entry en : e.getValue()) {
                if (en == null) continue;
                String line = line(en, 0, 50000, "");
                System.out.println(line);
                String rk = en.version + ":" + ruleKey(en.info);
                if (seenRules.add(rk)) {
                    extras.add(line(en, 1, 50000, "-anticipated"));
                    extras.add(line(en, 0, 100000, "-s100k"));
                }
            }
        }
        extras.forEach(System.out::println);
    }

    /** Case-id suffix of a version code. */
    static String edition(int version) {
        switch (version) {
            case 10: return "app6d";
            case 11: return "d";
            case 15: return "e";
            case 16: return "app6e";
            default: throw new IllegalArgumentException("version " + version);
        }
    }

    static boolean inSets(int set) { for (int s : SETS) if (s == set) return true; return false; }
    static String entityOf(String basic) { return basic.substring(2); }
    static String ruleKey(MSInfo i) { return (i.getSymbolSet() == 25 ? "cm:" : "mo:") + DrawRules.getDrawRuleName(i.getDrawRule()); }

    static String sidc(int version, int set, int status, String entity) {
        return String.format("%02d03%02d%d000%s0000", version, set, status, entity);
    }

    static String line(Entry en, int status, int scale, String suffix) {
        MSInfo info = en.info;
        String key = ruleKey(info);
        double[][] pts = LAYOUTS.get(STANDARD_LAYOUT.getOrDefault(en.version + ":" + entityOf(en.basic), key));
        if (pts == null) {
            System.err.println("no layout for " + key + " " + en.basic);
            pts = LAYOUTS.get("cm:Line1");
        }
        if (pts.length < info.getMinPointCount() || pts.length > info.getMaxPointCount()) {
            System.err.println("layout size " + pts.length + " outside " + info.getMinPointCount() + ".."
                    + info.getMaxPointCount() + " for " + key + " " + en.basic + " v" + en.version);
        }
        Map<String, String> mods = new LinkedHashMap<>();
        String[] size = SIZE_MODS.get(key);
        if (size != null) for (String s : size) mods.put(s.substring(0, s.indexOf('=')), s.substring(s.indexOf('=') + 1));
        if (info.getModifiers() != null) {
            for (String m : info.getModifiers()) {
                if (m == null || m.equals("A_SYMBOL_ICON") || mods.containsKey(m)) continue;
                mods.put(m, TEXT.getOrDefault(m, "M"));
            }
        }
        StringBuilder pt = new StringBuilder();
        double minLon = 1e9, maxLon = -1e9, minLat = 1e9, maxLat = -1e9;
        for (double[] q : pts) {
            double lon = LON0 + q[0] / M_PER_DEG_LON, lat = LAT0 + q[1] / M_PER_DEG_LAT;
            if (pt.length() > 0) pt.append(' ');
            pt.append(String.format(Locale.ROOT, "%.6f,%.6f", lon, lat));
            minLon = Math.min(minLon, lon); maxLon = Math.max(maxLon, lon);
            minLat = Math.min(minLat, lat); maxLat = Math.max(maxLat, lat);
        }
        double reach = 0;
        if (size != null) {
            for (String s : size) {
                if (!s.startsWith("AM_DISTANCE=")) continue;
                for (String n : s.substring(s.indexOf('=') + 1).split(",")) reach = Math.max(reach, Double.parseDouble(n));
            }
        }
        double dLon = 0.05 + reach / M_PER_DEG_LON, dLat = 0.05 + reach / M_PER_DEG_LAT;
        String bbox = String.format(Locale.ROOT, "%.4f,%.4f,%.4f,%.4f", minLon - dLon, minLat - dLat, maxLon + dLon, maxLat + dLat);
        StringBuilder b = new StringBuilder();
        b.append(String.format("%02d%s-%s%s", info.getSymbolSet(), entityOf(en.basic), edition(en.version), suffix));
        b.append('\t').append(sidc(en.version, info.getSymbolSet(), status, entityOf(en.basic)));
        b.append('\t').append(pt).append('\t').append(scale).append('\t').append(bbox);
        for (Map.Entry<String, String> m : mods.entrySet()) b.append('\t').append(m.getKey()).append('=').append(m.getValue());
        return b.toString();
    }
}
