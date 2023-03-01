use std::collections::LinkedList;
use std::sync::Arc;

use crate::data_source::DataSource;

type VirtualAddress = usize;

pub const PAGE_SIZE: usize = 4096;

struct MapEntry {
    source: Arc<dyn DataSource>,
    offset: usize,
    span: usize,
    addr: usize,
    flags: FlagBuilder,
}

impl MapEntry {
    #[must_use]
    pub fn new(source: Arc<dyn DataSource>, offset: usize, span: usize, addr: usize, flags: FlagBuilder) -> MapEntry {
      MapEntry {
        source: source.clone(),
        offset,
        span,
        addr,
        flags,
      }
    }
}

/// An address space.
pub struct AddressSpace {
    name: String,
    mappings: LinkedList<MapEntry>, // see below for comments
}

// comments about storing mappings
// Most OS code uses doubly-linked lists to store sparse data structures like
// an address space's mappings.
// Using Rust's built-in LinkedLists is fine. See https://doc.rust-lang.org/std/collections/struct.LinkedList.html
// But if you really want to get the zen of Rust, this is a really good read, written by the original author
// of that very data structure: https://rust-unofficial.github.io/too-many-lists/

// So, feel free to come up with a different structure, either a classic Rust collection,
// from a crate (but remember it needs to be #no_std compatible), or even write your own.
// See this ticket from Riley: https://github.com/dylanmc/cs393_vm_api/issues/10

impl AddressSpace {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            mappings: LinkedList::new(),
        }
    }

    fn round_up(addr: VirtualAddress) -> VirtualAddress {
        let floor = addr / PAGE_SIZE;
        if floor * PAGE_SIZE == addr {
          addr
        } else {
          (floor + 1) * PAGE_SIZE
        }
    }

    /// Add a mapping from a `DataSource` into this `AddressSpace`.
    ///
    /// # Errors
    /// If the desired mapping is invalid.
    pub fn add_mapping<D: DataSource + 'static>(
        &mut self,
        source: Arc<D>,
        offset: usize,
        span: usize,
        flags: FlagBuilder,
    ) -> Result<VirtualAddress, &str> {
        let span = Self::round_up(span);
        let mut curs = self.mappings.cursor_front_mut();
        let empty: bool = curs.current().is_none(); // curs starts pointing at first entry, only None if LL is empty
        while curs.current().is_some() {
          let this_ending: usize = {
            let entry = curs.current().expect("Bad things are happening.");
            entry.addr + entry.span
          };
          let next_address = if curs.peek_next().is_some() {
            curs.peek_next().expect("Bad things are happening.").addr
          } else {
            usize::MAX
          };
          //println!("{}",this_ending);
          //println!("{}",next_address);
          if next_address - this_ending >= span + 2 * PAGE_SIZE {
            break;
          }
          curs.move_next();
        }
        let next_addr: usize = if curs.peek_next().is_some() {
          let entry = curs.peek_next().expect("Bad things are happening.");
          entry.addr
        } else {
          usize::MAX // What is the size of the address space? Max usize? 
        };
        let this_ending: usize = if curs.current().is_some() {
          let entry = curs.current().expect("Bad things are happening.");
          entry.addr + entry.span
        } else {
          0
        };
        if next_addr - this_ending >= span + 2 * PAGE_SIZE || empty {
          let address = MapEntry::new(
            source,
            offset,
            span,
            this_ending + PAGE_SIZE,
            flags
          );
          curs.insert_after(address);
          Ok(this_ending + PAGE_SIZE)
        } else {
          println!("{}",this_ending);
          println!("{}",next_addr);
          Err("No memory chunk available.")
        }
    }

    /// Add a mapping from `DataSource` into this `AddressSpace` starting at a specific address.
    ///
    /// # Errors
    /// If there is insufficient room subsequent to `start`.
    pub fn add_mapping_at<D: DataSource + 'static>(
        &mut self,
        source: Arc<D>,
        offset: usize,
        span: usize,
        start: VirtualAddress,
        flags: FlagBuilder
    ) -> Result<(), &str> {
        let span = Self::round_up(span);
        let mut curs = self.mappings.cursor_front_mut();
        let empty: bool = curs.current().is_none();
        while curs.current().is_some() {
          let last_addr = {
            let entry = curs.current().expect("Bad things are happening.");
            entry.addr
          };
          let next_addr = if curs.peek_next().is_some() {
            curs.peek_next().expect("Bad things are happening.").addr
          } else {
            usize::MAX
          };
          if next_addr > start && last_addr < start {
            break;
          }
          curs.move_next();
        }
        let next_start = match curs.peek_next() {
          Some(x) => x.addr,
          None => usize::MAX,
        };
        let prev_end = match curs.current() {
          Some(x) => x.addr + x.span,
          None => 0,
        };
        if prev_end >= start - PAGE_SIZE || (next_start < start + span + PAGE_SIZE && !empty) {
          println!("{}",prev_end);
          println!("{}",next_start);
          Err("Insufficient free memory in desired region.") 
        } else {
          let new_map = MapEntry::new(
            source,
            offset,
            span,
            start,
            flags
          );
          curs.insert_after(new_map);
          Ok(())
        }
    }

    /// Remove the mapping to `DataSource` that starts at the given address.
    ///
    /// # Errors
    /// If the mapping could not be removed.
    pub fn remove_mapping<D: DataSource>(
        &mut self,
        source: Arc<D>,
        start: VirtualAddress,
    ) -> Result<(), &str> {
        let mut curs = self.mappings.cursor_front_mut();
        while curs.current().is_some() {
          let this_mapping = curs.current().expect("Bad things are happening.");
          if this_mapping.addr == start {
            break;
          }
          curs.move_next();
        }
        let mapping = curs.current();
        if mapping.is_none() ||  mapping.unwrap().addr != start {
          Err("No mapping with target address.")
        } else {
          curs.remove_current();
          Ok(()) //Do we have to drop a reference??
        }
    }

    /// Look up the DataSource and offset within that DataSource for a
    /// VirtualAddress / AccessType in this AddressSpace
    /// 
    /// # Errors
    /// If this VirtualAddress does not have a valid mapping in &self,
    /// or if this AccessType is not permitted by the mapping
    pub fn get_source_for_addr<D: DataSource>(
        &self,
        addr: VirtualAddress,
        access_type: FlagBuilder
    ) -> Result<(Arc<dyn DataSource>, usize), &str> {
        let mapping = self.get_mapping_for_addr(addr).expect("No mapping with target address.");
        let but_not_flags = access_type.but_not(mapping.flags);
        let any_disallowed = but_not_flags.read || but_not_flags.write || but_not_flags.execute || but_not_flags.cow || but_not_flags.private || but_not_flags.shared;
        match any_disallowed {
          true => Err("Given access type is not allowed for the data source at target address."),
          false => Ok((mapping.source.clone(), mapping.offset)),
        }
    }

    /// Helper function for looking up mappings
    fn get_mapping_for_addr(&self, addr: VirtualAddress) -> Result<&MapEntry, &str> {
        let mut curs = self.mappings.cursor_front();
        while curs.current().is_some() && curs.current().expect("Bad things are happening.").addr < addr {
          curs.move_next();
        }
        match curs.current() {
          Some(addr) => Ok(curs.current().expect("Bad things are happening.")),
          _ => Err("No mapping with target address."),
        }
    }
}

/// Build flags for address space maps.
///
/// We recommend using this builder type as follows:
/// ```
/// # use reedos_address_space::FlagBuilder;
/// let flags = FlagBuilder::new()
///     .toggle_read()
///     .toggle_write();
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // clippy is wrong: bools are more readable than enums
                                         // here because these directly correspond to yes/no
                                         // hardware flags
pub struct FlagBuilder {
    // TODO: should there be some sanity checks that conflicting flags are never toggled? can we do
    // this at compile-time? (the second question is maybe hard)
    read: bool,
    write: bool,
    execute: bool,
    cow: bool,
    private: bool,
    shared: bool,
}

impl FlagBuilder {
    pub fn check_access_perms(&self, access_perms: FlagBuilder) -> bool {
        if access_perms.read && !self.read || access_perms.write && !self.write || access_perms.execute && !self.execute {
            return false;
        }    
        true    
    }

    pub fn is_valid(&self) -> bool {
        if self.private && self.shared {
            return false;
        }
        if self.cow && self.write { // for COW to work, write needs to be off until after the copy
            return false;
        }
        true
    }
}

/// Create a constructor and toggler for a `FlagBuilder` object. Will capture attributes, including documentation
/// comments and apply them to the generated constructor.
macro_rules! flag {
    (
        $flag:ident,
        $toggle:ident
    ) => {
        #[doc=concat!("Turn on only the ", stringify!($flag), " flag.")]
        #[must_use]
        pub fn $flag() -> Self {
            Self {
                $flag: true,
                ..Self::default()
            }
        }

        #[doc=concat!("Toggle the ", stringify!($flag), " flag.")]
        #[must_use]
        pub const fn $toggle(self) -> Self {
            Self {
                $flag: !self.$flag,
                ..self
            }
        }
    };
}

impl FlagBuilder {
    /// Create a new `FlagBuilder` with all flags toggled off.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    flag!(read, toggle_read);
    flag!(write, toggle_write);
    flag!(execute, toggle_execute);
    flag!(cow, toggle_cow);
    flag!(private, toggle_private);
    flag!(shared, toggle_shared);

    #[must_use]
    /// Combine two `FlagBuilder`s by boolean or-ing each of their flags.
    ///
    /// This is, somewhat counter-intuitively, named `and`, so that the following code reads
    /// correctly:
    ///
    /// ```
    /// # use reedos_address_space::FlagBuilder;
    /// let read = FlagBuilder::read();
    /// let execute = FlagBuilder::execute();
    /// let new = read.and(execute);
    /// assert_eq!(new, FlagBuilder::new().toggle_read().toggle_execute());
    /// ```
    pub const fn and(self, other: Self) -> Self {
        let read = self.read || other.read;
        let write = self.write || other.write;
        let execute = self.execute || other.execute;
        let cow = self.cow || other.cow;
        let private = self.private || other.private;
        let shared = self.shared || other.shared;

        Self {
            read,
            write,
            execute,
            cow,
            private,
            shared,
        }
    }

    #[must_use]
    /// Turn off all flags in self that are on in other.
    ///
    /// You can think of this as `self &! other` on each field.
    ///
    /// ```
    /// # use reedos_address_space::FlagBuilder;
    /// let read_execute = FlagBuilder::read().toggle_execute();
    /// let execute = FlagBuilder::execute();
    /// let new = read_execute.but_not(execute);
    /// assert_eq!(new, FlagBuilder::new().toggle_read());
    /// ```
    pub const fn but_not(self, other: Self) -> Self {
        let read = self.read && !other.read;
        let write = self.write && !other.write;
        let execute = self.execute && !other.execute;
        let cow = self.cow && !other.cow;
        let private = self.private && !other.private;
        let shared = self.shared && !other.shared;

        Self {
            read,
            write,
            execute,
            cow,
            private,
            shared,
        }
    }
}
