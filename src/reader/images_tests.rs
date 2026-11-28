use std::path::Path;

use super::*;

#[test]
fn a_lone_image_reads_its_alt_and_destination() {
    let image = parse("![Marauder map](maps/marauder.png)").expect("an image");
    assert_eq!(image, Image { alt: "Marauder map".into(), target: "maps/marauder.png".into() });
    assert_eq!(parse("![](a.png)").map(|image| image.alt), Some(String::new()));
    assert_eq!(parse("![a](<my map.png>)").map(|image| image.target), Some("my map.png".into()));
    assert_eq!(parse("![a](b(c).png)").map(|image| image.target), Some("b(c).png".into()));
}

#[test]
fn a_title_after_the_destination_is_allowed_but_not_other_text() {
    let titled = parse("![Marauder map](maps/marauder.png \"The Marauder's Map\")").expect("an image");
    assert_eq!(titled.target, "maps/marauder.png");
    assert!(parse("![a](b.png 'c')").is_some() && parse("![a](b.png (c))").is_some());
    assert!(parse("![a](b.png c)").is_none(), "an unquoted word after the destination is not a title");
}

#[test]
fn a_sentence_a_link_an_html_image_and_two_images_are_not_one_image() {
    for text in
        ["see ![x](y) here", "[link](y)", "<img src=\"y\">", "![x](y) and more", "![x](y)![z](w)", "![x]", "![x]()"]
    {
        assert_eq!(parse(text), None, "{text}");
    }
}

#[test]
fn brackets_nest_in_the_alt_text_and_an_escape_does_not_close_it() {
    assert_eq!(parse("![a [b] c](x.png)").map(|image| image.alt), Some("a [b] c".into()));
    assert_eq!(parse("![a \\] b](x.png)").map(|image| image.alt), Some("a \\] b".into()));
}

#[test]
fn a_fence_carries_its_image_and_only_its_image() {
    let line = "  ![Marauder map](maps/marauder.png)  ";
    assert_eq!(opening(line), "```fmc-image ![Marauder map](maps/marauder.png)");
    assert_eq!(from_info(opening(line).trim_start_matches("```")), parse(line.trim()));
    assert_eq!(from_info("mermaid"), None);
    assert_eq!(from_info("fmc-image"), None);
    assert_eq!(from_info("rust ![a](b.png)"), None);
}

#[test]
fn a_destination_is_classified_for_the_box() {
    assert_eq!(classify("https://hogwarts.test/map.png"), Source::Remote);
    assert_eq!(classify("//hogwarts.test/map.png"), Source::Remote);
    assert_eq!(classify("images/seal.SVG"), Source::Svg);
    assert_eq!(classify("maps/marauder.png"), Source::Local("maps/marauder.png".into()));
    assert_eq!(classify("Marauder%20map.png"), Source::Local("Marauder map.png".into()));
    assert_eq!(classify("notes/vault.txt"), Source::Other);
    assert_eq!(classify("data:image/png;base64,AAAA"), Source::Other);
}

#[test]
fn a_destination_that_climbs_out_of_the_root_is_not_inside_it() {
    let root = Path::new("/vault");
    let folder = Path::new("/vault/Gringotts");
    assert_eq!(inside(root, folder, "../Louvre/mona.png"), Some("/vault/Louvre/mona.png".into()));
    assert_eq!(inside(root, folder, "maps/./marauder.png"), Some("/vault/Gringotts/maps/marauder.png".into()));
    assert_eq!(inside(root, folder, "../../etc/passwd.png"), None);
    assert_eq!(inside(root, folder, "/elsewhere/seal.png"), None);
    assert_eq!(inside(root, folder, "/vault/CERN/atlas.png"), Some("/vault/CERN/atlas.png".into()));
}

#[test]
fn an_image_name_is_its_alt_or_its_file_name() {
    let named = Image { alt: " Marauder map ".into(), target: "maps/marauder.png".into() };
    assert_eq!(named.label(), "Marauder map");
    let unnamed = Image { alt: String::new(), target: "maps/Marauder%20map.png".into() };
    assert_eq!(unnamed.label(), "Marauder map.png");
}

#[test]
fn a_percent_escape_that_is_not_two_hex_digits_stays_as_written() {
    assert_eq!(decoded("a%20b"), "a b");
    assert_eq!(decoded("a%zz%2"), "a%zz%2");
}
