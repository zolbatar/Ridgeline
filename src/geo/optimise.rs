use geos::{GResult, Geom, Geometry};

pub fn union_linestrings(lines: &[Geometry]) -> GResult<Geometry> {
    let mut union_geom = geos::Geom::clone(&lines[0]);
    for line in lines.iter().skip(1) {
        union_geom = union_geom.union(line)?;
    }
    Ok(union_geom)
}
