use ca_formats::{
    apgcode::ApgCode,
    plaintext::Plaintext,
    rle::{HeaderData, Rle},
};
use std::{error::Error, fs::File};

#[test]
fn rle_sirrobin() -> Result<(), Box<dyn Error>> {
    let file = File::open("tests/sirrobin.rle")?;
    let sirrobin = Rle::new_from_file(file)?;

    assert_eq!(
        sirrobin.header_data(),
        Some(&HeaderData {
            x: 31,
            y: 79,
            rule: Some(String::from("B3/S23"))
        })
    );

    assert_eq!(sirrobin.count(), 282);

    Ok(())
}

#[test]
fn plaintext_sirrobin() -> Result<(), Box<dyn Error>> {
    let file = File::open("tests/sirrobin.cells")?;
    let sirrobin = Plaintext::new_from_file(file)?;

    assert_eq!(sirrobin.count(), 282);

    Ok(())
}

#[test]
fn apgcode_sirrobin() -> Result<(), Box<dyn Error>> {
    let apgcode = "xq6_yyocxukcy6gocs20h0a38bac2qq73uszyjo4w8y0e4mo0vu0o606s6444u08clav0h03g440qq1333333x11zy9ecec2ik032i210sw3f0hy011w70401011033547442zy0emj896he1e1kif6q2gc50ew9qb30dzgo403gg066m32w11z34407q441n6zy311";
    let sirrobin = ApgCode::new(apgcode)?;

    assert_eq!(sirrobin.period(), 6);

    assert_eq!(sirrobin.count(), 290);

    Ok(())
}
