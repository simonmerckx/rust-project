//! File system with directory support
//!
//! Create a filesystem that has a notion of blocks, inodes and directory inodes, by implementing the [`FileSysSupport`], the [`BlockSupport`], the [`InodeSupport`] and the [`DirectorySupport`] traits together (again, all earlier traits are supertraits of the later ones).
//!
//! [`FileSysSupport`]: ../../cplfs_api/fs/trait.FileSysSupport.html
//! [`BlockSupport`]: ../../cplfs_api/fs/trait.BlockSupport.html
//! [`InodeSupport`]: ../../cplfs_api/fs/trait.InodeSupport.html
//! [`DirectorySupport`]: ../../cplfs_api/fs/trait.DirectorySupport.html
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

use cplfs_api::{controller::Device, error_given, fs::{BlockSupport, DirectorySupport, FileSysSupport, InodeSupport}, types::{Block, DIRENTRY_SIZE, DIRNAME_SIZE, DirEntry, FType, Inode, SuperBlock}};
use thiserror::Error;
use crate::b_inode_support::{self, CustomInodeFileSystem};

/// You are free to choose the name for your file system. As we will use
/// automated tests when grading your assignment, indicate here the name of
/// your file system data type so we can just use `FSName` instead of
/// having to manually figure out the name.
/// **TODO**: replace the below type by the type of your file system
pub type FSName = CustomDirFileSystem;

// Custom type
/// Custom file system data type
pub struct CustomDirFileSystem {
    //device: Device,
    inode_fs: CustomInodeFileSystem,
}

impl CustomDirFileSystem {

    /// Create a new InodeCustomFileSystem given a BlockCustomFileSystem
    pub fn new(inodefs: CustomInodeFileSystem) -> CustomDirFileSystem {
        CustomDirFileSystem {  inode_fs: inodefs }
    }  
}

#[derive(Error, Debug)]
/// Custom type for errors in my implementation
pub enum CustomDirFileSystemError {
    /// An error occured in the inode layer
    #[error("InodeFileSystemError")]
    GivenError(#[from] b_inode_support::CustomInodeFileSystemError),
    #[error("API error")]
    /// The input provided to some method in the controller layer was invalid
    APIError(#[from] error_given::APIError),
    #[error("No directory entry was found for the provided name")]
    /// The provided name did not correspond to a directory entry
    NoEntryFoundForName,
    #[error("The provided inode is not of directory type")]
    /// The provided inode to search for a directory is not of the right type
    InodeWrongType,
    #[error("The provided name is invalid for a directory entry")]
    /// The provided name is invalid for a directory entry
    InvalidEntryName


}

impl FileSysSupport for CustomDirFileSystem {
    type Error = CustomDirFileSystemError;

    fn sb_valid(sb: &SuperBlock) -> bool {
        return CustomInodeFileSystem::sb_valid(sb);
    }
    fn mkfs<P: AsRef<std::path::Path>>(path: P, sb: &SuperBlock) -> Result<Self, Self::Error> {
        let mut inode_fs = CustomInodeFileSystem::mkfs(path, sb)?;
        // get the first inode and change it's nlink attribute
        let mut root_inode = inode_fs.i_get(1)?;
        root_inode.disk_node.nlink = 1;
        // Change type
        root_inode.disk_node.ft = FType::TDir;
        inode_fs.i_put(&root_inode)?;
        return Ok(CustomDirFileSystem::new(inode_fs))
    }

    fn mountfs(dev: Device) -> Result<Self, Self::Error> {
        let inode_fs = CustomInodeFileSystem::mountfs(dev)?;
        return Ok(CustomDirFileSystem::new(inode_fs));
    }

    fn unmountfs(self) -> Device {
        return self.inode_fs.unmountfs();
    }
}


impl BlockSupport for CustomDirFileSystem {
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

impl InodeSupport for CustomDirFileSystem {
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

impl DirectorySupport for CustomDirFileSystem {
    fn new_de(inum: u64, name: &str) -> Option<DirEntry> {
        let mut dir_entry = DirEntry::default();
        dir_entry.inum = inum;
        match Self::set_name_str(&mut dir_entry, name) {
            Some(x) => return Some(dir_entry),
            None => return None
        }
    }

    fn get_name_str(de: &DirEntry) -> String {
        let char_array = de.name;
        let mut string = String::new();
        for i in 0..DIRNAME_SIZE {
            if char_array[i] == '\0' {
                break
            }
            else {
                string.push(char_array[i]);
            }
        }
        return string;
    }

    fn set_name_str(de: &mut DirEntry, name: &str) -> Option<()> {
        let empty_cond = name.is_empty();
        let point_cond = !(name == "." || name == ".." || name.chars().all(char::is_alphanumeric));
        let length_cond = name.len() > DIRNAME_SIZE;
        if empty_cond || point_cond || length_cond{
            return None
        }
        else {
            let mut newname = name.to_string();
            if newname.len() < DIRNAME_SIZE {
                newname.push('\0');
            } 
            let chars: Vec<char> = newname.chars().collect();
            let mut array = ['\0'; DIRNAME_SIZE];
            let mut index = 0;
            for i in chars {
                array[index] = i;  
                index = index + 1;
            }
            de.name = array;
            return Some(())
        } 
    }

    fn dirlookup(&self, inode: &Self::Inode, name: &str) -> Result<(Self::Inode, u64), Self::Error> {
        if !(inode.disk_node.ft == FType::TDir) {
            return Err(CustomDirFileSystemError::InodeWrongType);
        }
        let superblock = self.sup_get()?;
        let file_blocks = inode.disk_node.direct_blocks;
        let nb_selected_blocks = (inode.disk_node.size as f64/superblock.block_size as f64).ceil(); 
        for index in 0..(nb_selected_blocks as u64) {
            let element = file_blocks[index as usize];
            if !(element == 0) {
                let block = self.b_get(element - superblock.datastart)?;
                let nb_dirs = superblock.block_size/ *DIRENTRY_SIZE;
                let mut offset = 0 ;
                for _ in 0..(nb_dirs) {
                    let dir_entry = block.deserialize_from::<DirEntry>(offset)?;
                    // check if this is not an empty entry
                    if dir_entry.inum != 0 {
                        // check if the names match
                        if Self::get_name_str(&dir_entry) == *name {
                            let inode = self.i_get(dir_entry.inum)?;
                            return Ok((inode, superblock.block_size*index + offset))
                        }
                    }
                    offset += *DIRENTRY_SIZE;
                }
            }
        }

        return Err(CustomDirFileSystemError::NoEntryFoundForName)
    }

    fn dirlink(&mut self,inode: &mut Self::Inode,name: &str,inum: u64,) -> Result<u64, Self::Error> {
        if !(inode.disk_node.ft == FType::TDir) {
            return Err(CustomDirFileSystemError::InodeWrongType);
        }
        
        let new_dir_entry = match Self::new_de(inum,name) {
            None => return Err(CustomDirFileSystemError::InvalidEntryName),
            Some(dir_entry) => dir_entry
        };

        let superblock = self.sup_get()?;
        let file_blocks = inode.disk_node.direct_blocks;
        let nb_selected_blocks = (inode.disk_node.size as f64/superblock.block_size as f64).ceil(); 
        for index in 0..(nb_selected_blocks as u64) {
            let element = file_blocks[index as usize];
            if !(element == 0) {
                let mut block = self.b_get(element - superblock.datastart)?;
                let nb_dirs = superblock.block_size/ *DIRENTRY_SIZE;
                let mut offset = 0 ;
                for _ in 0..(nb_dirs) {
                    let dir_entry = block.deserialize_from::<DirEntry>(offset)?;
                    // check if we have an empty entry
                    if dir_entry.inum == 0 {
                        block.serialize_into(&new_dir_entry, offset)?;
                        let mut inode = self.i_get(inum)?;
                        if !(inode.inum == inum) {
                            inode.disk_node.nlink += 1;
                            self.i_put(&inode)?;
                            return Ok(superblock.block_size*index + offset)
                        }     
                    }
                offset += *DIRENTRY_SIZE;

                }
            }
        }

        //  In case the directory has no free entries,
        //  append a new entry to the end and increase the size of inode. 
        // no free dir entry was found

        return Ok(1)
        


        
    }
}


// **TODO** define your own tests here.

// WARNING: DO NOT TOUCH THE BELOW CODE -- IT IS REQUIRED FOR TESTING -- YOU WILL LOSE POINTS IF I MANUALLY HAVE TO FIX YOUR TESTS
#[cfg(all(test, any(feature = "c", feature = "all")))]
#[path = "../../api/fs-tests/c_test.rs"]
mod tests;
