#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("noder.hpp");

        type GeometryCollectionShim = crate::cxxbridge::GeometryCollectionShim;

        fn _compile_tester(geoms: &GeometryCollectionShim);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_tester() {
        let collection = geo::GeometryCollection::default();
        let collection = crate::cxxbridge::GeometryCollectionShim(collection);
        ffi::_compile_tester(&collection);
    }
}
