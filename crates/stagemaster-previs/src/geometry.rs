use crate::Triangle;
use stagemaster_spatial::polygon::floor_plan;

pub(super) fn prism(
    mesh: &mut Vec<Triangle>,
    outline: &[[f64; 2]],
    bottom: f64,
    top: f64,
) -> Result<(), String> {
    let plan = floor_plan(outline)?;
    for triangle in plan.triangles {
        let [a, b, c] = triangle;
        mesh.push([[a[0], a[1], top], [b[0], b[1], top], [c[0], c[1], top]]);
        mesh.push([
            [a[0], a[1], bottom],
            [c[0], c[1], bottom],
            [b[0], b[1], bottom],
        ]);
    }
    let area_twice = outline
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let b = outline[(i + 1) % outline.len()];
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>();
    for (i, a) in outline.iter().enumerate() {
        let b = outline[(i + 1) % outline.len()];
        let mut one = [
            [a[0], a[1], bottom],
            [b[0], b[1], bottom],
            [b[0], b[1], top],
        ];
        let mut two = [[a[0], a[1], bottom], [b[0], b[1], top], [a[0], a[1], top]];
        if area_twice < 0.0 {
            one.swap(1, 2);
            two.swap(1, 2);
        }
        mesh.push(one);
        mesh.push(two);
    }
    Ok(())
}
pub(super) fn wall(
    mesh: &mut Vec<Triangle>,
    a: [f64; 2],
    b: [f64; 2],
    thickness: f64,
    bottom: f64,
    top: f64,
) {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let len = dx.hypot(dy);
    let normal = [-dy / len * thickness / 2.0, dx / len * thickness / 2.0];
    let outline = [
        [a[0] + normal[0], a[1] + normal[1]],
        [a[0] - normal[0], a[1] - normal[1]],
        [b[0] - normal[0], b[1] - normal[1]],
        [b[0] + normal[0], b[1] + normal[1]],
    ];
    // Short/thin valid walls can be below the editable floor-area minimum. Construct the
    // rectangular solid directly instead of applying a user polygon's area policy.
    let [a, b, c, d] = outline;
    let at = |p: [f64; 2], z: f64| [p[0], p[1], z];
    mesh.extend([
        [at(a, top), at(b, top), at(c, top)],
        [at(a, top), at(c, top), at(d, top)],
        [at(a, bottom), at(c, bottom), at(b, bottom)],
        [at(a, bottom), at(d, bottom), at(c, bottom)],
    ]);
    for (i, a) in outline.iter().enumerate() {
        let b = outline[(i + 1) % 4];
        mesh.push([at(*a, bottom), at(b, bottom), at(b, top)]);
        mesh.push([at(*a, bottom), at(b, top), at(*a, top)]);
    }
}
