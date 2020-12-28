//! File system with inode support + read and write operations on inodes
//!
//! Create a filesystem that has a notion of inodes and blocks, by implementing the [`FileSysSupport`], the [`BlockSupport`] and the [`InodeSupport`] traits together (again, all earlier traits are supertraits of the later ones).
//! Additionally, implement the [`InodeRWSupport`] trait to provide operations to read from and write to inodes
//!
//! [`FileSysSupport`]: ../../cplfs_api/fs/trait.FileSysSupport.html
//! [`BlockSupport`]: ../../cplfs_api/fs/trait.BlockSupport.html
//! [`InodeSupport`]: ../../cplfs_api/fs/trait.InodeSupport.html
//! [`InodeRWSupport`]: ../../cplfs_api/fs/trait.InodeRWSupport.html
//! Make sure this file does not contain any unaddressed `TODO`s anymore when you hand it in.
//!
//! # Status
//!
//! **TODO**: Replace the question mark below with YES, NO, or PARTIAL to
//! indicate the status of this assignment. If you want to tell something
//! about this assignment to the grader, e.g., you have a bug you can't fix,
//! or you want to explain your approach, write it down after the comments
//! section. If you had no major issues and everything works, there is no need to write any comments.
//!
//! COMPLETED: PARTIAL
//!
//! COMMENTS:
//!
//! ...
//!

use thiserror::Error;
use cplfs_api::{controller::Device, error_given::{self, APIError}, fs::{BlockSupport, FileSysSupport, InodeRWSupport, InodeSupport}, types::{Block, Buffer, Inode, SuperBlock}};

use crate::b_inode_support::{self, CustomInodeFileSystem};

/// You are free to choose the name for your file system. As we will use
/// automated tests when grading your assignment, indicate here the name of
/// your file system data type so we can just use `FSName` instead of
/// having to manually figure out the name.
/// **TODO**: replace the below type by the type of your file system
pub type FSName = CustomInodeRWFileSystem;

// Custom type
/// Custom file system data type
pub struct CustomInodeRWFileSystem {
    inode_fs: CustomInodeFileSystem,
}

impl CustomInodeRWFileSystem {

    /// Create a new InodeCustomFileSystem given a BlockCustomFileSystem
    pub fn new(inodefs: CustomInodeFileSystem) -> CustomInodeRWFileSystem {
        CustomInodeRWFileSystem {  inode_fs: inodefs }
    }  
}

#[derive(Error, Debug)]
/// Custom type for errors in my implementation
pub enum CustomInodeRWFileSystemError {
    /// An error occured in the inode layer
    #[error("InodeFileSystemError")]
    GivenError(#[from] b_inode_support::CustomInodeFileSystemError),
    /// The provided index is larger than the size of the file
    #[error("The provided index is larger than the size of the file")]
    IndexOutOfBounds,
    #[error("API error")]
    /// The input provided to some method in the controller layer was invalid
    APIError(#[from] error_given::APIError),
}


impl FileSysSupport for CustomInodeRWFileSystem {
    type Error = CustomInodeRWFileSystemError;

    fn sb_valid(sb: &SuperBlock) -> bool {
        return CustomInodeFileSystem::sb_valid(sb);
    }
    fn mkfs<P: AsRef<std::path::Path>>(path: P, sb: &SuperBlock) -> Result<Self, Self::Error> {
        let inode_fs = CustomInodeFileSystem::mkfs(path, sb)?;
        return Ok(CustomInodeRWFileSystem::new(inode_fs))
    }

    fn mountfs(dev: Device) -> Result<Self, Self::Error> {
        let inode_fs = CustomInodeFileSystem::mountfs(dev)?;
        return Ok(CustomInodeRWFileSystem::new(inode_fs));
    }

    fn unmountfs(self) -> Device {
        return self.inode_fs.unmountfs();
    }
}

impl BlockSupport for CustomInodeRWFileSystem {
    fn b_get(&self, i: u64) -> Result<Block, Self::Error> {
        let block = self.inode_fs.b_get(i)?;
        return Ok(block);      
    }

    fn b_put(&mut self, b: &Block) -> Result<(), Self::Error> {
        let result = self.inode_fs.b_put(b)?;
        return Ok(result);
    }

    fn b_free(&mut self, i: u64) -> Result<(), Self::Error> {
        let res = self.inode_fs.b_free(i)?;
        return Ok(res)       
    }

    fn b_zero(&mut self, i: u64) -> Result<(), Self::Error> {
        let result = self.inode_fs.b_zero(i)?;
        return Ok(result);
    }

    fn b_alloc(&mut self) -> Result<u64, Self::Error> {
        let index = self.inode_fs.b_alloc()?;
        return Ok(index);
    }

    fn sup_get(&self) -> Result<SuperBlock, Self::Error> {
        let superblock = self.inode_fs.sup_get()?;
        return Ok(superblock);
    }

    fn sup_put(&mut self, sup: &SuperBlock) -> Result<(), Self::Error> {
        let result = self.inode_fs.sup_put(sup)?;
        return Ok(result);
    }
}

impl InodeSupport for CustomInodeRWFileSystem {
    type Inode = Inode;

    fn i_get(&self, i: u64) -> Result<Self::Inode, Self::Error> {
        let inode = self.inode_fs.i_get(i)?;
        return Ok(inode);
    }

    fn i_put(&mut self, ino: &Self::Inode) -> Result<(), Self::Error> {
        let result = self.inode_fs.i_put(ino)?;
        return Ok(result);
    }

    fn i_free(&mut self, i: u64) -> Result<(), Self::Error> {
        let result = self.inode_fs.i_free(i)?;
        return Ok(result);
    }

    fn i_alloc(&mut self, ft: cplfs_api::types::FType) -> Result<u64, Self::Error> {
        let i = self.inode_fs.i_alloc(ft)?;
        return Ok(i);
    }

    fn i_trunc(&mut self, inode: &mut Self::Inode) -> Result<(), Self::Error> {
        let result = self.inode_fs.i_trunc(inode)?;
        return Ok(result);
    }
}

impl InodeRWSupport for CustomInodeRWFileSystem {
    fn i_read(&self, inode: &Self::Inode, buf: &mut Buffer, off: u64, n: u64) -> Result<u64, Self::Error> {
        // If a read starts at inode.get_size(), returns with 0 bytes read.
        if off == inode.disk_node.size {
            return Ok(0);
        }
        // returns an error and does not read anything if index falls further outside of the file's bounds. 
        if off > inode.disk_node.size {
            return Err(CustomInodeRWFileSystemError::IndexOutOfBounds);
        }

        let superblock = self.sup_get()?;
        let file_blocks = inode.disk_node.direct_blocks;
        let nb_selected_blocks = (inode.disk_node.size as f64/superblock.block_size as f64).ceil(); 
        let mut buf_offset = 0;
        for index in 0..(nb_selected_blocks as u64) {
            // skip the blocks that don't contain bytes we need
            if (index +1)*superblock.block_size < off {
                continue
            }
            // we only want to read n bytes, also stop if buf is full
            if buf_offset >= n || buf_offset >= buf.len() {
                break
            }
            let element = file_blocks[index as usize];
            if !(element == 0) {
                // b-get: read the nth block of the entire disk and return it
                let block = self.b_get(element)?;
                //let mut offset = 0;
                for byte_index in 0..(superblock.block_size) {
                    // we only want to read n bytes and stop when end of file is reached
                    if buf_offset >= n || buf_offset >= inode.disk_node.size {
                        break
                    };
                    // start reading from byte offset off in the inode 
                    if index * superblock.block_size + byte_index >= off {
                        let mut byte: [u8;1] = [0];
                        block.read_data(&mut byte, byte_index)?;
                        // If buf cannot hold n bytes of data, reads until buf is full instead.
                        match buf.write_data(&byte, buf_offset) {
                            // reached end of the buf stop adding
                            Err(APIError::BlockInput("Trying to write beyond the bounds of the block",)) => break,
                            // not specified what to do in other cases
                            Err(_) => (),
                            Ok(_) => ()
                        }
                        buf_offset += 1;
                    }               
                }
            }
        }

        return Ok(buf_offset);
    }

    fn i_write(&mut self,inode: &mut Self::Inode,buf: &cplfs_api::types::Buffer,off: u64, n: u64) -> Result<(), Self::Error> {
        todo!()
    }
}


// **TODO** define your own tests here.

// WARNING: DO NOT TOUCH THE BELOW CODE -- IT IS REQUIRED FOR TESTING -- YOU WILL LOSE POINTS IF I MANUALLY HAVE TO FIX YOUR TESTS
#[cfg(all(test, any(feature = "e", feature = "all")))]
#[path = "../../api/fs-tests/e_test.rs"]
mod tests;
