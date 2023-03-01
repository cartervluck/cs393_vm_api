#![feature(linked_list_cursors)]

#![allow(dead_code, unused_variables)]

mod address_space;
mod cacher;
mod data_source;

pub use address_space::{AddressSpace, FlagBuilder};
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
        let read_flags = FlagBuilder::new().toggle_read();

        let ds_arc = Arc::new(data_source);

        let addr = addr_space.add_mapping(ds_arc.clone(), offset, length, read_flags).unwrap();
        assert!(addr != 0);

        let addr2 = addr_space.add_mapping(ds_arc.clone(), address_space::PAGE_SIZE, length, read_flags).unwrap();
        assert!(addr2 != 0);
        assert!(addr != addr2);
        
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
        let read_flags = FlagBuilder::new().toggle_read();

        let ds_arc = Arc::new(data_source);

        let addr = addr_space.add_mapping_at(ds_arc.clone(), offset, length, address_space::PAGE_SIZE + 1, read_flags);
        match addr {
          Ok(_) => println!("First address added successfully."),
          Err(e) => panic!("{}", e),
        }

        let addr2 = addr_space.add_mapping_at(ds_arc.clone(), address_space::PAGE_SIZE, length, 3 * address_space::PAGE_SIZE + 3, read_flags);
        match addr2 {
          Ok(_) => println!("Second address added successfully."),
          Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn consec_mapping_at_failure() {
        let mut addr_space = AddressSpace::new("Test address space");
        let data_source: FileDataSource = FileDataSource::new("Cargo.toml").unwrap();
        let offset: usize = 0;
        let length: usize = 1;
        let read_flags = FlagBuilder::new().toggle_read();

        let ds_arc = Arc::new(data_source);

        let addr = addr_space.add_mapping_at(ds_arc.clone(), offset, length, address_space::PAGE_SIZE + 1, read_flags);
        match addr {
          Ok(_) => println!("First address added successfully."),
          Err(e) => panic!("{}", e),
        }

        let addr2 = addr_space.add_mapping_at(ds_arc.clone(), address_space::PAGE_SIZE, length, address_space::PAGE_SIZE + 1, read_flags);
        assert!(addr2.is_err())
    }   

    #[test]
    fn consec_mapping_at_with_remove() {
        let mut addr_space = AddressSpace::new("Test address space");
        let data_source: FileDataSource = FileDataSource::new("Cargo.toml").unwrap();
        let offset: usize = 0;
        let length: usize = 1;
        let read_flags = FlagBuilder::new().toggle_read();

        let ds_arc = Arc::new(data_source);

        let addr = addr_space.add_mapping_at(ds_arc.clone(), offset, length, address_space::PAGE_SIZE + 1, read_flags);
        match addr {
          Ok(_) => println!("First address added successfully."),
          Err(e) => panic!("{}", e),
        }

        let r = addr_space.remove_mapping(ds_arc.clone(), address_space::PAGE_SIZE + 1);
        match r {
          Ok(_) => println!("First address removed successfully."),
          Err(e) => panic!("{}", e),
        }

        let addr2 = addr_space.add_mapping_at(ds_arc.clone(), address_space::PAGE_SIZE, length, 3 * address_space::PAGE_SIZE + 1, read_flags);
        match addr2 {
          Ok(_) => println!("Second address added successfully."),
          Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn mapping_end() {
        let mut addr_space = AddressSpace::new("Test address space");
        let data_source: FileDataSource = FileDataSource::new("Cargo.toml").unwrap();
        let offset: usize = 0;
        let length: usize = 1;
        let read_flags = FlagBuilder::new().toggle_read();

        let ds_arc = Arc::new(data_source);

        let addr = addr_space.add_mapping_at(ds_arc.clone(), offset, length, address_space::PAGE_SIZE + 1, read_flags);
        match addr {
          Ok(_) => println!("First address added successfully."),
          Err(e) => panic!("{}", e),
        }

        let addr2 = addr_space.add_mapping_at(ds_arc.clone(), address_space::PAGE_SIZE, length, usize::MAX - 2 * address_space::PAGE_SIZE - 1, read_flags);
        match addr2 {
          Ok(_) => println!("Second address added successfully."),
          Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn get_source_valid() {
        let mut addr_space = AddressSpace::new("Test address space");
        let data_source = FileDataSource::new("Cargo.toml").unwrap();
        let data_source2 = FileDataSource::new("README.md").unwrap();
        let offset: usize = 0;
        let length: usize = 1;
        let read_flags = FlagBuilder::new().toggle_read();

        let fds_arc = Arc::new(data_source);
        let fds_arc2 = Arc::new(data_source2);

        let ds_arc: Arc<dyn DataSource> = fds_arc.clone();

        let ds_arc2: Arc<dyn DataSource> = fds_arc2.clone();

        let addr = addr_space.add_mapping_at(fds_arc.clone(), offset, length, address_space::PAGE_SIZE + 1, read_flags);
        match addr {
          Ok(_) => println!("First address added successfully."),
          Err(e) => panic!("{}", e),
        }

        let addr2 = addr_space.add_mapping_at(fds_arc2.clone(), offset, length, 3 * address_space::PAGE_SIZE + 3, read_flags);
        match addr2 {
          Ok(_) => println!("Second address added successfully."),
          Err(e) => panic!("{}", e),
        }

        let result = addr_space.get_source_for_addr::<FileDataSource>(address_space::PAGE_SIZE + 1, read_flags);
        match result {
          Ok((source_result, offset_result)) => println!("TODO: Arc comparison"),
          Err(e) => panic!("{}", e),
        }
 
        let result2 = addr_space.get_source_for_addr::<FileDataSource>(3 * address_space::PAGE_SIZE + 3, read_flags);
        match result2 {
          Ok((source_result, offset_result)) => println!("TODO: Arc comparison"),
          Err(e) => panic!("{}", e),
        }
    }
}
