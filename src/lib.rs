#![allow(dead_code, unused_variables)]

mod address_space;
mod cacher;
mod data_source;

pub use address_space::AddressSpace;
pub use data_source::{DataSource, FileDataSource};
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors() {
        let _a = AddressSpace::new("my first address space");
        let _ds: FileDataSource = FileDataSource::new("Cargo.toml").unwrap(); // a little silly, but why not?
    }

    // more tests here - add mappings, read data, remove mappings and add more, make sure the
    // address space has what we expect in it after each operation

    // test if mapping has been added
    #[test]
    fn test_add_mapping() {
        let mut addr_space = AddressSpace::new("Test address space");
        let data_source: FileDataSource = FileDataSource::new("Cargo.toml").unwrap();
        let offset: usize = 0;
        let length: usize = 1;

        let ds_arc = Arc::new(data_source);

        let addr = addr_space.add_mapping(ds_arc.clone(), offset, length).unwrap();
        assert!(addr != 0);

        let addr2 = addr_space.add_mapping(ds_arc.clone(), address_space::PAGE_SIZE, length).unwrap();
        assert!(addr2 != 0);
        assert!(addr != addr2);
        
        println!("{}",addr);
        println!("{}",addr2);

        // we should move these tests into addr_space, since they access non-public internals of the structure:
        // assert_eq!(addr_space.mappings.is_empty(), false);
        // assert_eq!(addr_space.mappings.front().source, Some(&data_source));
        // assert_eq!(addr_space.mappings.front().offset, offset);
        // assert_eq!(addr_space.mappings.front().span, length);
    }

    #[test]
    fn add_mapping_at_correct() {
        let mut addr_space = AddressSpace::new("Test address space");
        let data_source: FileDataSource = FileDataSource::new("Cargo.toml").unwrap();
        let offset: usize = 0;
        let length: usize = 1;

        let ds_arc = Arc::new(data_source);

        let addr = addr_space.add_mapping_at(ds_arc.clone(), offset, length, address_space::PAGE_SIZE + 1);
        match addr {
          Ok(_) => println!("First address added successfully."),
          Err(e) => panic!("{}", e),
        }

        let addr2 = addr_space.add_mapping_at(ds_arc.clone(), address_space::PAGE_SIZE, length, 2 * address_space::PAGE_SIZE + 3);
        match addr2 {
          Ok(_) => println!("Second address added successfully."),
          Err(e) => panic!("{}", e),
        }
        // assert_eq!(addr_space.mappings.is_empty(), false);
        // assert_eq!(addr_space.mappings.front().source, Some(&data_source));
        // assert_eq!(addr_space.mappings.front().offset, offset);
        // assert_eq!(addr_space.mappings.front().span, length);
    }
}
