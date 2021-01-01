//! File system with block support
//!
//! Create a filesystem that only has a notion of blocks, by implementing the [`FileSysSupport`] and the [`BlockSupport`] traits together (you have no other choice, as the first one is a supertrait of the second).
//!
//! [`FileSysSupport`]: ../../cplfs_api/fs/trait.FileSysSupport.html
//! [`BlockSupport`]: ../../cplfs_api/fs/trait.BlockSupport.html
//! Make sure this file does not contain any unaddressed `TODO`s anymore when you hand it in.
//!
//! # Status
//!
//!
//! COMPLETED: YES
//!
//! COMMENTS:
//!
//! ...
//!

use std::path::Path;

// If you want to import things from the API crate, do so as follows:
use cplfs_api::{controller::Device, error_given, fs::FileSysSupport, types::{DINODE_SIZE}};
// import SuperBlock
use cplfs_api::types::SuperBlock;
// import BlockSupport
use cplfs_api::fs::BlockSupport;
use cplfs_api::types::{Block};

// use auxiliary package thiserror to make the definition of errors easier
use thiserror::Error;

/// Custom type 
pub type FSName = CustomBlockFileSystem;

/// Custom block file system data type
pub struct CustomBlockFileSystem {
    /// Device type representing the state of the hard drive disk 
    /// allows to read disk blocks from the disk, and write disk blocks to the disk
    pub device: Device, 
    /// Cached SuperBlock
    pub superblock: SuperBlock
}


impl CustomBlockFileSystem {
    /// Create a new CustomBlockFileSystem given a Device dev
    pub fn new(dev: Device, sb: SuperBlock) -> CustomBlockFileSystem {
        CustomBlockFileSystem { device: dev, superblock: sb }
    }  
}

#[derive(Error, Debug)]
/// Custom type for errors in CustomBlockFileSystem
pub enum CustomBlockFileSystemError {
    #[error("invalid SuperBlock was detected")]
    /// Error thrown when an invalid superblock is encountered
    InvalidSuperBlock,
    #[error("SuperBlock and device are not compatible")]
    /// Error thrown when the device and superblock don't agree
    IncompatibleDeviceSuperBlock,
    #[error("The data index is out of bounds for this device")]
    /// Thrown when the block index provided is larger than ndatablocks - 1
    DataIndexOutOfBounds,
    #[error("The block that was tried to be freed is already free")]
    /// Thrown when the block that is trying to be freed is already free
    BlockIsAlreadyFree,
    #[error("There is no free data block")]
    /// Thrown when there is no free data block available and one is requested 
    NoFreeDataBlock,
    /// The input provided to some method in the controller layer was invalid
    #[error("API error")]
    GivenError(#[from] error_given::APIError)
}

impl FileSysSupport for CustomBlockFileSystem {
    fn sb_valid(sb: &SuperBlock) -> bool {
        // the bitmap starts after the inodes
        if !(sb.inodestart < sb.bmapstart) {
            return false
        }
        // at least one block for the bit map
        if !(sb.bmapstart < sb.datastart) {
            return false
        }
        // One block for the Superblock
        let order_cond3 =  sb.inodestart > 0;  
        // The inode region has to be sufficiently large to hold ninodes inodes 
        let inode_cond =  *DINODE_SIZE * sb.ninodes <= (sb.bmapstart - sb.inodestart) * sb.block_size;
        // The bitmap needs to provide place for 1 bit for every datablock
        let hold_cond1 = (sb.datastart - sb.bmapstart) * sb.block_size * 8 >= sb.ndatablocks;
        // There needs to be enough space for the datablocks
        let hold_cond2 = sb.datastart + sb.ndatablocks <= sb.nblocks;
        // The regions have to physically fit on the disk together, i.e. fall within the first nblocks blocks
        let fit_cond1 = 1 + (sb.bmapstart - sb.inodestart) + (sb.datastart - sb.bmapstart) + sb.ndatablocks <= sb.nblocks;
        if order_cond3 && hold_cond1 && hold_cond2 && inode_cond && fit_cond1 {
            return true
        }
        else {
            return false
        }  
    }

    fn mkfs<P: AsRef<Path>>(path: P, sb: &SuperBlock) -> Result<Self, Self::Error>{
        // Check if the given superblock is a valid file system superblock
        let sb_cond = Self::sb_valid(sb);
        if !sb_cond {
            return Err(CustomBlockFileSystemError::InvalidSuperBlock);
        } else  {
           //Create a new Device at the given path, to allow the file system to communicate with it
           let mut device = Device::new(path, sb.block_size, sb.nblocks)?;
           // A super block containing the file system metadata at block index 0
           let mut block = device.read_block(0)?;
           block.serialize_into(sb, 0)?;
           // write this block to the device
           device.write_block(&block)?;
           return Ok(CustomBlockFileSystem::new(device, *sb));
        }     
    }

    fn mountfs(dev: Device) -> Result<Self, Self::Error> {
        // The superblock is a valid superblock 
        let sb_block = dev.read_block( 0)?;
        let superblock = sb_block.deserialize_from::<SuperBlock>(0)?;
        if Self::sb_valid(&superblock) {
            // The block size and number of blocks of the device and superblock agree
            if dev.block_size == superblock.block_size && dev.nblocks == superblock.nblocks {
                return Ok(CustomBlockFileSystem::new(dev, superblock))
            }
            else {
                return Err(CustomBlockFileSystemError::IncompatibleDeviceSuperBlock);
            }            
        }
        else {
            return Err(CustomBlockFileSystemError::InvalidSuperBlock);
        }
    }

    fn unmountfs(self) -> Device {
        return self.device
    }

    type Error = CustomBlockFileSystemError;
}

impl BlockSupport for CustomBlockFileSystem {
    //Read the nth block of the entire disk and return it
    fn b_get(&self, i: u64) -> Result<Block, Self::Error> {
        let block = self.device.read_block(i)?;
        return Ok(block)
    }

    //Write the nth block of the entire disk and return it
    fn b_put(&mut self, b: &Block) -> Result<(), Self::Error> {
        let block = self.device.write_block(b)?;
        return Ok(block);
    }

    // Free the ith block in the block data region, by setting the ith bit in the free bit map region to zero.
    fn b_free(&mut self, i: u64) -> Result<(), Self::Error> {
        let superblock = self.sup_get()?;
        // Index i is out of bounds, if it's higher than the number of data blocks
        if i > superblock.ndatablocks - 1 {
            return Err(CustomBlockFileSystemError::DataIndexOutOfBounds);
        }
        // bitmap can be mutiple blocks large, we have to select the right one
        let bitmapblockcapacity = superblock.block_size * 8;
        let block_offset = i / bitmapblockcapacity;
        let mut bitmap_block = self.b_get(superblock.bmapstart + block_offset)?;
        // one byte of data
        let mut byte: [u8; 1] = [0];
        // the byte we want to read from the bitmap block
        let byte_offset =  (i % bitmapblockcapacity) / 8;
        bitmap_block.read_data(&mut byte, byte_offset)?;
        // because << adds zeros we should do this and invert later
        let bit_offset =  (i % bitmapblockcapacity) % 8;
        let set_byte = 0b0000_0001 << bit_offset;
        // we define the order of the bits within each byte you read from right to left
        let or = byte[0] | !set_byte;
        if or == !set_byte {
            // ith block is already a free block
            return Err(CustomBlockFileSystemError::BlockIsAlreadyFree);
        }
        else{
            let and = byte[0] & !set_byte;
            let res = bitmap_block.write_data(&[and], byte_offset)?;
            self.b_put(&bitmap_block)?;
            return Ok(res)
        }    
    }

    fn b_zero(&mut self, i: u64) -> Result<(), Self::Error> {
        let superblock = self.sup_get()?;
        // Index i is out of bounds, if it is higher than the number of data blocks
        if i > superblock.ndatablocks - 1 {
            return Err(CustomBlockFileSystemError::DataIndexOutOfBounds)
        }
        self.b_put(&Block::new_zero(superblock.datastart + i, superblock.block_size))
        
    }

    fn b_alloc(&mut self) -> Result<u64, Self::Error> {
        let superblock = self.sup_get()?;
        let nbbitmapblocks = superblock.datastart - superblock.bmapstart;
        for x in 0..nbbitmapblocks {
            let mut bitmap_block = self.b_get(superblock.bmapstart + x)?;
            for y in 0..superblock.block_size {
                let mut byte: [u8; 1] = [0];
                bitmap_block.read_data(&mut byte, y)?;
                for z in 0..8 {
                    let set_byte = 0b0000_0001 << z;
                    let and = byte[0] & set_byte;
                    // This spot is free so we can use it
                    if !(and == set_byte) {
                        let index = (x*superblock.block_size*8) + (y*8) + z;
                        // The bitmap only consists of ndatablock bits,
                        // if we go past this we are looking in a part of the last
                        // bitmap block that is not allocated for the bitmap
                        if index > superblock.ndatablocks - 1{
                            return Err(CustomBlockFileSystemError::NoFreeDataBlock);  
                        } 
                        let new_byte = byte[0] | set_byte;
                        bitmap_block.write_data(&[new_byte], y)?;
                        self.b_put(&bitmap_block)?;
                        self.b_zero(index)?;
                        return Ok(index)
                    }
                }    
            }
        }
        // nothing changed
        return Err(CustomBlockFileSystemError::NoFreeDataBlock);     
    }

    fn sup_get(&self) -> Result<SuperBlock, Self::Error> {
        return Ok(self.superblock);
    }

    fn sup_put(&mut self, sup: &SuperBlock) -> Result<(), Self::Error> {
        let mut block = self.b_get(0)?;
        block.serialize_into( sup, 0)?;
        self.b_put(&block)?;
        self.superblock = *sup;
        return Ok(())
    }
}


// Here we define a submodule, called `my_tests`, that will contain your unit
// tests for this module.
// **TODO** define your own tests here. I have written down one test as an example of the syntax.
// You can define more tests in different modules, and change the name of this module
//
// The `test` in the `#[cfg(test)]` annotation ensures that this code is only compiled when we're testing the code.
// To run these tests, run the command `cargo test` in the `solution` directory
//
// To learn more about testing, check the Testing chapter of the Rust
// Book: https://doc.rust-lang.org/book/testing.html
#[cfg(test)]
mod my_tests {

    #[test]
    fn trivial_unit_test() {
        assert_eq!(2, 2);
        assert!(true);
    }
}

// If you want to write more complicated tests that create actual files on your system, take a look at `utils.rs` in the assignment, and how it is used in the `fs_tests` folder to perform the tests. I have imported it below to show you how it can be used.
// The `utils` folder has a few other useful methods too (nothing too crazy though, you might want to write your own utility functions, or use a testing framework in rust, if you want more advanced features)
#[cfg(test)]
#[path = "../../api/fs-tests"]
mod test_with_utils {
    use std::path::PathBuf;

    use cplfs_api::{fs::{BlockSupport, FileSysSupport}, types::SuperBlock};
    use super::CustomBlockFileSystem;
    //use a_block_support::CustomBlockFileSystem;
    fn disk_prep_path(name: &str) -> PathBuf {
        utils::disk_prep_path(&("fs-images-a-".to_string() + name), "img")
    }
    

    #[path = "utils.rs"]
    mod utils;

    #[test]
    fn sb_valid() {
        static BLOCK_SIZE: u64 = 1000;
        static NBLOCKS: u64 = 10;
        static SUPERBLOCK_BAD: SuperBlock = SuperBlock {
            block_size: BLOCK_SIZE, //Note; assumes at least 2 inodes fit in one block. 
            nblocks: NBLOCKS,
            ninodes: 6,
            inodestart: 1,
            // too many data blocks
            ndatablocks: 20,
            bmapstart: 4,
            datastart: 5,
        };

        // Too many data blocks for the bitmap to store
        static SUPERBLOCK_BAD_2: SuperBlock = SuperBlock {
            block_size: 120, //Note; assumes at least 1  inodes fit in one block.
            nblocks: 1000,
            ninodes: 3,
            inodestart: 1,
            // too many data blocks
            ndatablocks: 995,
            bmapstart: 4,
            datastart: 5,
        };

        assert_eq!(CustomBlockFileSystem::sb_valid(&SUPERBLOCK_BAD), false);
        assert_eq!(CustomBlockFileSystem::sb_valid(&SUPERBLOCK_BAD_2), false);
    }

    #[test]
    fn free_alloc_multiple_bblocks() {
        static SUPERBLOCK_GOOD: SuperBlock = SuperBlock {
            block_size: 300, //Note; assumes at least 1 inodes fit in one block.
            nblocks: 2500,
            ninodes: 3,
            inodestart: 1,
            ndatablocks: 2494,
            bmapstart: 4,
            datastart: 6,
        };

        assert_eq!(CustomBlockFileSystem::sb_valid(&SUPERBLOCK_GOOD), true);   
        let path = disk_prep_path("free_alloc_multiple_blocks");
        let mut my_fs = CustomBlockFileSystem::mkfs(&path, &SUPERBLOCK_GOOD).unwrap();
        let mut byte: [u8; 1] = [0];
        for i in 0..SUPERBLOCK_GOOD.ndatablocks {
            assert_eq!(my_fs.b_alloc().unwrap(), i); //Fill up all data blocks
        }
        assert!(my_fs.b_alloc().is_err()); //No more blocks

        //Check the bitmap in second block
        let bb = my_fs.b_get(5).unwrap();
        bb.read_data(&mut byte, 0).unwrap();
        // everything here should be allocated in first byte
        assert_eq!(byte[0], 0b1111_1111);
        
        //Deallocate fist block
        my_fs.b_free(2400).unwrap();
        assert!(my_fs.b_free(2400).is_err());

        //Check the bitmap in second block
        let bb = my_fs.b_get(5).unwrap();
        bb.read_data(&mut byte, 0).unwrap();
        assert_eq!(byte[0], 0b1111_1110);

        let dev = my_fs.unmountfs();
        utils::disk_destruct(dev);
    }
    
    #[test]
    fn unit_test() {
        //The below method set up the parent folder "a_parent_unique_name" within the root directory  of this solution crate
        //Also delete the file "image_file" within this folder if it already exists, so that it does not interfere with any later `mkfs` calls (this is useful if your previous test run failed, and the file did not get deleted)
        //*WARNING* !Make sure that this folder name "a_parent_unique_name" is actually unique over different tests, because tests are executed in parallel by default!
        //Returns the concatenated path, so that you can use the path further on, e.g. when creating a `Device` or `FileSystem`

        //! `let path = utils::disk_prep_path("a_parent_unique_name", "image_file");`

        //Things you want to test go here (check my tests in the API folder for examples)
        //! ...
        //! ...

        // If some disk actually created the file under `path` in your code, then you can uncomment the following call to clean it up:
        //!  `utils::disk_unprep_path(&path);`
        // This removes the image file and the parent directory at the end, so that no garbage is left in your file system
        //*WARNING* if a Device `dev` is still in scope for the path `path`, then the above call will block (the device holds a lock on the memory-mapped file)
        //You then have to use the following call instead:

        //! `utils::disk_destruct(dev);`

        //This makes the device go out of scope first, before tearing down the parent folder and image file, thereby avoiding deadlock
    }
}

// Here we define a submodule, called `tests`, that will contain our unit tests
// Take a look at the specified path to figure out which tests your code has to pass.
// As with all other files in the assignment, the testing module for this file is stored in the API crate (this is the reason for the 'path' attribute in the code below)
// The reason I set it up like this is that it allows me to easily add additional tests when grading your projects, without changing any of your files, but you can still run my tests together with yours by specifying the right features (see below) :)
// directory.
//
// To run these tests, run the command `cargo test --features="X"` in the `solution` directory, with "X" a space-separated string of the features you are interested in testing.
//
// WARNING: DO NOT TOUCH THE BELOW CODE -- IT IS REQUIRED FOR TESTING -- YOU WILL LOSE POINTS IF I MANUALLY HAVE TO FIX YOUR TESTS
//The below configuration tag specifies the following things:
// 'cfg' ensures this module is only included in the source if all conditions are met
// 'all' is true iff ALL conditions in the tuple hold
// 'test' is only true when running 'cargo test', not 'cargo build'
// 'any' is true iff SOME condition in the tuple holds
// 'feature = X' ensures that the code is only compiled when the cargo command includes the flag '--features "<some-features>"' and some features includes X.
// I declared the necessary features in Cargo.toml
// (Hint: this hacking using features is not idiomatic behavior, but it allows you to run your own tests without getting errors on mine, for parts that have not been implemented yet)
// The reason for this setup is that you can opt-in to tests, rather than getting errors at compilation time if you have not implemented something.
// The "a" feature will run these tests specifically, and the "all" feature will run all tests.
#[cfg(all(test, any(feature = "a", feature = "all")))]
#[path = "../../api/fs-tests/a_test.rs"]
mod tests;
