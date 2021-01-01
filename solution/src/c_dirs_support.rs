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
//!
//! COMPLETED: YES
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
    inode_fs: CustomInodeFileSystem,
}

impl CustomDirFileSystem {

    /// Create a new CustomDirFileSystem given a CustomInodeFileSystem
    pub fn new(inodefs: CustomInodeFileSystem) -> CustomDirFileSystem {
        CustomDirFileSystem {  inode_fs: inodefs }
    }  
}

#[derive(Error, Debug)]
/// Custom type for errors in CustomDirFileSystem
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
    InvalidEntryName,
    #[error("Inode corresponding to inum is not currently in use")]
    /// Inode corresponding to inum is not currently in use.
    DirectoryInodeNotInUse,
    #[error("Inode has no room for extra block")]
    /// Inode has no room for extra block
    InodeBlocksFull

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
            Some(_) => return Some(dir_entry),
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
                index += 1;
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
                // b-get: read the nth block of the entire disk and return it
                let block = self.b_get(element)?;
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
                    if offset >= inode.disk_node.size {
                        break;
                    }
                }
            }
        }

        return Err(CustomDirFileSystemError::NoEntryFoundForName)
    }

    fn dirlink(&mut self,inode: &mut Self::Inode,name: &str,inum: u64,) -> Result<u64, Self::Error> {
        // The inode has to be a directory
        if !(inode.disk_node.ft == FType::TDir) {
            return Err(CustomDirFileSystemError::InodeWrongType);
        }

        let mut corresponding_inode = self.i_get(inum)?;
        // errors and does nothing if the inode corresponding to inum is not currently in use.
        if corresponding_inode.disk_node.ft == FType::TFree {
            return Err(CustomDirFileSystemError::DirectoryInodeNotInUse);
        };

        //name is invalid
        let new_dir_entry = match Self::new_de(inum,name) {
            None => return Err(CustomDirFileSystemError::InvalidEntryName),
            Some(dir_entry) => dir_entry
        };

        // Name is already an entry inside inode.
        match self.dirlookup(inode, name) {
            // the name already exists, so return error
            Ok(_)=> return Err(CustomDirFileSystemError::InvalidEntryName),
            Err(_) => ()
        }

        let superblock = self.sup_get()?;
        let file_blocks = inode.disk_node.direct_blocks;
        let nb_selected_blocks = (inode.disk_node.size as f64/superblock.block_size as f64).ceil(); 
        for index in 0..(nb_selected_blocks as u64) {
            let element = file_blocks[index as usize];
            if !(element == 0) {
                // b-get: read the nth block of the entire disk and return it
                let mut block = self.b_get(element)?;
                let nb_dirs = superblock.block_size/ *DIRENTRY_SIZE;
                let mut offset = 0 ;
                for _ in 0..(nb_dirs) {
                    let dir_entry = block.deserialize_from::<DirEntry>(offset)?;
                    // check if we have an empty entry
                    // we might be over the size of the inode
                    // but there might still place in this block 
                    // to add a dir entry
                    if dir_entry.inum == 0 || offset >= inode.disk_node.size {
                        if offset >= inode.disk_node.size {
                            inode.disk_node.size += *DIRENTRY_SIZE;
                            self.i_put(&inode)?;
                        }
                        block.serialize_into(&new_dir_entry, offset)?;  
                        // write block back to disk
                        self.b_put(&block)?;
                        // if inum and inode's number are equal, then nothing happens
                        if !(inode.inum == inum) {
                            corresponding_inode.disk_node.nlink += 1;
                            self.i_put(&corresponding_inode)?;      
                        } 
                        return Ok(superblock.block_size*index + offset);
                    }
                    offset +=  *DIRENTRY_SIZE;           
                }
            }
        }

        // inode has no room for extra block
        if nb_selected_blocks == inode.disk_node.direct_blocks.len() as f64 {
            return Err(CustomDirFileSystemError::InodeBlocksFull);
        }

        // if we did not exit the function
        // allocate a new block
        // Returns the index (within the data region) of the newly allocated block.
        let new_block_index = superblock.datastart + self.b_alloc()?;
        let mut new_block = self.b_get(new_block_index)?;
        // we start at the beginning of the block
        new_block.serialize_into(&new_dir_entry, 0)?;  
        // increase the size
        inode.disk_node.size += *DIRENTRY_SIZE;
        // find zero element and change it with index

        
        
        inode.disk_node.direct_blocks[nb_selected_blocks as usize] = new_block_index;
        // write inode back
        self.i_put(inode)?;
        // put the block back on disk
        self.b_put(&new_block)?;
        corresponding_inode = self.i_get(inum)?;
        if !(corresponding_inode.inum == inum) {
            corresponding_inode.disk_node.nlink += 1;
            self.i_put(&corresponding_inode)?;      
        } 
        return Ok(inode.disk_node.size -  *DIRENTRY_SIZE);       
    }
}



#[cfg(test)]
#[path = "../../api/fs-tests"]
mod test_with_utils {
    use std::path::PathBuf;

    fn disk_prep_path(name: &str) -> PathBuf {
        utils::disk_prep_path(&("fs-images-a-".to_string() + name), "img")
    }

    #[path = "utils.rs"]
    mod utils;
}

// WARNING: DO NOT TOUCH THE BELOW CODE -- IT IS REQUIRED FOR TESTING -- YOU WILL LOSE POINTS IF I MANUALLY HAVE TO FIX YOUR TESTS
#[cfg(all(test, any(feature = "c", feature = "all")))]
#[path = "../../api/fs-tests/c_test.rs"]
mod tests;
