use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use stagemaster_project::{Document, EditCommand, PatchReport};

fn fixture() -> Document {
    let mut root: Value = serde_json::from_slice(include_bytes!(
        "../../../docs/project-format/examples/lighting-basic.project.json"
    ))
    .unwrap();
    root["entryPoints"] = json!([]);
    Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap()
}
fn rows(report: &PatchReport) -> Vec<csv::StringRecord> {
    csv::Reader::from_reader(report.bytes())
        .records()
        .collect::<Result<_, _>>()
        .unwrap()
}
#[test]
fn complete_identity_patch_and_exact_snapshot_are_read_only_and_deterministic() {
    let d = fixture();
    let before = d.encode().unwrap();
    let report = d.patch_report().unwrap();
    assert_eq!(report.fixture_count(), 1);
    assert!(PatchReport::recognizes(report.bytes()));
    assert!(report.bytes().starts_with(b"\xef\xbb\xbf"));
    assert!(report.bytes().ends_with(b"\r\n"));
    let rows = rows(&report);
    let f = &d.view().fixtures[0];
    assert_eq!(&rows[0][4], format!("{:x}", Sha256::digest(&before)));
    assert_eq!(&rows[0][5], f.id);
    assert_eq!(&rows[0][6], f.name);
    assert_eq!(&rows[0][9], "4 通道");
    assert_eq!(
        (&rows[0][13], &rows[0][14], &rows[0][15], &rows[0][16]),
        ("1", "1", "4", "4")
    );
    assert_eq!(
        (&rows[0][17], &rows[0][18], &rows[0][21]),
        ("已配适", "未布置", "")
    );
    assert_eq!(&rows[0][27], "入口灯组");
    assert_eq!(report.bytes(), d.patch_report().unwrap().bytes());
    assert_eq!(before, d.encode().unwrap());
}
#[test]
fn unpatched_fixtures_are_kept_with_empty_numbers_and_zero_coordinates_are_not_missing() {
    let mut root: Value = serde_json::from_slice(&fixture().encode().unwrap()).unwrap();
    root["lighting"]["patches"] = json!([]);
    let mut d = Document::decode(&serde_json::to_vec(&root).unwrap()).unwrap();
    let id = d.view().fixtures[0].id.clone();
    d.edit(
        serde_json::from_value(
            json!({"op":"stage","command":{"op":"putPlacement","placement":{
              "fixtureId":id,"spaceId":null,"positionMeters":{"x":"0","y":"-2.5","z":"0"},
              "rotationDegreesXYZ":{"x":"0","y":"-30","z":"90"}
            }}}),
        )
        .unwrap(),
    )
    .unwrap();
    let row = rows(&d.patch_report().unwrap()).remove(0);
    assert_eq!((&row[13], &row[14], &row[15]), ("", "", ""));
    assert_eq!(
        (&row[17], &row[18], &row[19]),
        ("未配适", "已布置", "未归属空间")
    );
    assert_eq!((&row[21], &row[22], &row[23]), ("0", "-2.5", "0"));
    assert_eq!((&row[24], &row[25], &row[26]), ("0", "-30", "90"));
}
#[test]
fn csv_quotes_multiline_text_and_formula_like_names_without_changing_source() {
    let mut d = fixture();
    d.edit(EditCommand::SetInfo {
        name: "=工程,\"中文\"\n第二行".into(),
        description: String::new(),
    })
    .unwrap();
    let before = d.encode().unwrap();
    let report = d.patch_report().unwrap();
    assert_eq!(&rows(&report)[0][1], "'=工程,\"中文\"\n第二行");
    assert!(PatchReport::recognizes(report.bytes()));
    assert_eq!(before, d.encode().unwrap());
    assert!(!PatchReport::recognizes(b"name,address\r\nother,1\r\n"));
    assert!(!PatchReport::recognizes(&report.bytes()[..100]));
}
#[test]
fn address_order_is_numeric_and_empty_projects_export_a_valid_header() {
    let mut d = Document::new("地址排序").unwrap();
    let view = d.view();
    for address in [100, 5, 20] {
        d.edit(EditCommand::AddFixture {
            name: format!("灯{address}"),
            profile_id: view.profiles[1].id.clone(),
            domain_id: view.domains[0].id.clone(),
            universe: 1,
            address,
        })
        .unwrap();
    }
    let report = d.patch_report().unwrap();
    assert_eq!(
        rows(&report).iter().map(|r| &r[14]).collect::<Vec<_>>(),
        ["5", "20", "100"]
    );
    let empty = Document::new("空工程").unwrap().patch_report().unwrap();
    assert_eq!(empty.fixture_count(), 0);
    assert!(rows(&empty).is_empty());
    assert!(PatchReport::recognizes(empty.bytes()));
}

#[test]
fn space_and_rig_membership_come_from_world_placement_and_attachment() {
    let mut d = fixture();
    let id = d.view().fixtures[0].id.clone();
    d.edit(EditCommand::Stage {
        command: serde_json::from_value(json!({"op":"putSpace","id":null,"name":"主厅","outlineMeters":[["0","0"],["10","0"],["10","6"],["0","6"]],"floorElevationMeters":"0","clearHeightMeters":"7"})).unwrap(),
    }).unwrap();
    let room = d.view().stage.spaces[0].id.clone();
    d.edit(EditCommand::Stage {
        command: serde_json::from_value(json!({"op":"putConstruction","id":null,"name":"前桁架","shape":{"kind":"rig","rigKind":"truss","spaceId":room,"positionMeters":{"x":"4","y":"3","z":"5"},"yawDegrees":"0","lengthMeters":"6","widthMeters":"0.3","heightMeters":"0.4"}})).unwrap(),
    }).unwrap();
    let view = d.view();
    d.edit(
        serde_json::from_value(json!({"op":"stage","command":{
          "op":"attachFixtures","constructionId":view.stage.constructions[0].id,"fixtureIds":[id],
          "layout":{"startMarginMeters":"1","endMarginMeters":"1","dropMeters":"0.2"}
        }}))
        .unwrap(),
    )
    .unwrap();
    let row = rows(&d.patch_report().unwrap()).remove(0);
    assert_eq!(
        (&row[19], &row[20], &row[21], &row[22], &row[23]),
        ("主厅", "前桁架", "4", "3", "4.6")
    );
}
