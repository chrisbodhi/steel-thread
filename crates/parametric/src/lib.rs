use domain::ActuatorPlate;
use validation;

pub trait Validation {
    // TODO: figure out how to mesh `plate` arg here with generic trait
    // TODO: We may want a T that matches ValidationError when we define this trait for real
    fn is_valid(plate: ActuatorPlate) -> Result<(), ValidationError>;
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    NoStep,
}

pub fn generate_step(plate: ActuatorPlate) -> Result<(), ValidationError> {
    if let Err(e) = validation::validate(&plate) {
        eprintln!("oops: {}", e);
        return Err(ValidationError::NoStep);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use domain::Millimeters;

    use super::*;

    #[test]
    fn it_fails_when_it_should() {
        let mut plate = ActuatorPlate::default();
        plate.bolt_diameter = Millimeters(0);
        let err = Err(ValidationError::NoStep);
        assert_eq!(err, generate_step(plate))
    }
}
