use choochoo_resource::Profile;
use choochoo_rt_model::Destination;

#[test]
fn profile_defaults_to_default() -> Result<(), Box<dyn std::error::Error>> {
    let dest = Destination::<()>::builder().build()?;

    assert_eq!(Profile::DEFAULT_STR, &**dest.profile());

    Ok(())
}

#[test]
fn profile_uses_custom_specified() -> Result<(), Box<dyn std::error::Error>> {
    let dest = Destination::<()>::builder()
        .with_profile(Profile::new("custom")?)
        .build()?;

    assert_eq!("custom", &**dest.profile());

    Ok(())
}
