use reliakit_primitives::{BoundedStr, ByteSize, NonEmptyStr, Percent, Port};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    type ServiceName = BoundedStr<3, 32>;

    let display_name = NonEmptyStr::new("Reliakit API")?;
    let service_name = ServiceName::new("api-service")?;
    let threshold = Percent::new(95)?;
    let port = Port::new(8080)?;
    let body_limit = ByteSize::from_mb(10);

    println!("display name: {display_name}");
    println!("service name: {service_name}");
    println!("threshold: {threshold}");
    println!("port: {port}");
    println!("body limit: {body_limit}");

    Ok(())
}
