use assert_cmd::Command;

#[test]
fn pdf_signature_on_unsigned_pdf_emits_zero_count() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    {
        use lopdf::{dictionary, Document, Object, Stream};
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Courier",
        });
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! { "F1" => font_id },
        });
        let content = lopdf::content::Content {
            operations: vec![
                lopdf::content::Operation::new("BT", vec![]),
                lopdf::content::Operation::new("Tf", vec!["F1".into(), 12.into()]),
                lopdf::content::Operation::new("Td", vec![100.into(), 100.into()]),
                lopdf::content::Operation::new("Tj", vec![Object::string_literal("hello")]),
                lopdf::content::Operation::new("ET", vec![]),
            ],
        };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        doc.save(tmp.path()).unwrap();
    }
    let out = Command::cargo_bin("ubertool")
        .unwrap()
        .args(["--json", "pdf", "signature", "--in"])
        .arg(tmp.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");
    assert_eq!(v["signature_count"], 0);
    assert!(v["signatures"].as_array().unwrap().is_empty());
}

#[test]
fn pdf_signature_missing_file_exits_4() {
    Command::cargo_bin("ubertool")
        .unwrap()
        .args(["pdf", "signature", "--in", "/nonexistent/path.pdf"])
        .assert()
        .failure()
        .code(4);
}
