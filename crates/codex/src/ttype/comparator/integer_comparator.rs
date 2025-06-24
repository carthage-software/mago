use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::union::TUnion;

pub fn is_contained_by_union(input_type_part: TInteger, container_type: &TUnion) -> bool {
    let int_containers: Vec<TInteger> = container_type
        .types
        .iter()
        .filter_map(
            |atomic| if let TAtomic::Scalar(TScalar::Integer(integer)) = atomic { Some(*integer) } else { None },
        )
        .collect();

    input_type_part.is_contained_by_any(int_containers)
}
