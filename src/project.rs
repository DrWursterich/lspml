use lsp_types::Url;

pub(crate) struct WorkingDirectory {
    pub module: String,
    pub path: String,
}

pub(crate) fn get_working_directory(uri: &Url) -> Option<WorkingDirectory> {
    let path = uri.to_file_path().ok()?.to_str()?.to_owned();
    // assume directory above "src" is the module
    let index = path.find("/src/")?;
    let path = &path[..index];
    let begin = path.rfind('/')? + 1;
    return Some(WorkingDirectory {
        module: path[begin..].to_string(),
        path: path[..begin].to_string(),
    });
}
