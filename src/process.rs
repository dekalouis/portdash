use std::{collections::HashMap, fs, io};

/// Scans `/proc/<pid>/fd` and maps each socket inode to the PIDs holding it.
///
/// Processes that cannot be read (other users, or already exited) are skipped,
/// so their sockets simply have no entry. Fails only if `/proc` itself is unreadable.
pub fn socket_owners() -> io::Result<HashMap<u64, Vec<u32>>> {
    let mut found: HashMap<u64, Vec<u32>> = HashMap::new();
    let entries = fs::read_dir("/proc")?;
    for entry in entries {
        // One unreadable entry should not stop the whole scan.
        let Ok(entry) = entry else { continue };
        let name = entry.file_name();
        let Some(text) = name.to_str() else { continue };

        let Ok(pid) = text.parse::<u32>() else {
            continue;
        };
        let pid_folder = format!("/proc/{pid}/fd");

        let Ok(fd_dir) = fs::read_dir(pid_folder) else {
            continue;
        };

        for fd_entry in fd_dir {
            let Ok(fd_entry) = fd_entry else { continue };
            let Ok(target) = fs::read_link(fd_entry.path()) else {
                continue;
            };
            let Some(target) = target.to_str() else {
                continue;
            };
            let Some(inode) = socket_inode(target) else {
                continue;
            };
            let list = found.entry(inode).or_default();
            if !list.contains(&pid) {
                list.push(pid);
            }
        }
    }
    Ok(found)
}

fn socket_inode(link_target: &str) -> Option<u64> {
    let rest = link_target.strip_prefix("socket:[")?;
    let digits = rest.strip_suffix("]")?;
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_socket_inode() {
        assert_eq!(socket_inode("socket:[123]"), Some(123));
    }

    #[test]
    fn rejects_dev_null() {
        assert_eq!(socket_inode("/dev/null"), None);
    }

    #[test]
    fn rejects_non_numeric_inode() {
        assert_eq!(socket_inode("socket:[abc]"), None);
    }

    #[test]
    fn rejects_a_pipe() {
        assert_eq!(socket_inode("pipe:[123]"), None);
    }
}
