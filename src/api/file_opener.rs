use crate::*;


#[deprecated = "Use FileOpener instead"]
pub type FileSender<'a, R> = FileOpener<'a, R>;


/// API of opening file/dir with other apps.
/// 
/// # Examples
/// ```
/// fn example(app: &tauri::AppHandle) {
///     use tauri_plugin_android_fs::AndroidFsExt as _;
/// 
///     let api = app.android_fs();
///     let file_sender = api.file_sender();
/// }
/// ```
pub struct FileOpener<'a, R: tauri::Runtime>(
    #[allow(unused)]
    pub(crate) &'a AndroidFs<R>
);

impl<'a, R: tauri::Runtime> FileOpener<'a, R> {

    /// Show app chooser for sharing files with other apps.   
    /// This function returns immediately after requesting to open the app chooser, 
    /// without waiting for the app’s response. 
    /// 
    /// This sends the files as a single unit.
    /// The available apps depend on the MIME types associated with the files.  
    /// This does not result in an error even if no available apps are found. 
    /// An empty app chooser is displayed.
    /// 
    /// # Args
    /// - ***uris*** :  
    /// Target file URIs to share.  
    /// This all needs to be **readable**.  
    /// URIs converted directly from a path, such as via [`FileUri::from_path`], can **not** be used.   
    /// 
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_SEND_MULTIPLE>
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_SEND>
    pub fn share_files<'b>(
        &self, 
        uris: impl IntoIterator<Item = &'b FileUri>, 
    ) -> crate::Result<()> {

        // もし use_app_chooser と exclude_self_from_app_chooser を関数の引数として公開するなら、
        //  Show app chooser for sharing files with other apps.   
        //. This function returns immediately after requesting to open the app chooser, 
        // を以下に変更した方が説明文としてわかりやすいかもしれない。
        //  Share files with other app that user selected. 
        //  This function returns immediately after requesting to share the files, 

        on_android!({
            impl_se!(struct Req<'a> { uris: Vec<&'a FileUri>, common_mime_type: Option<&'a str>, use_app_chooser: bool, exclude_self_from_app_chooser: bool });
            impl_de!(struct Res;);

            // Decides whether the app chooser dialog is always shown.  
            // The recommended value is true, which ensures the dialog is always displayed.  
            // If set to false, the behavior depends on the user’s previous choice: 
            // if the user has previously selected an app as "ALWAYS" in Android, 
            // that app will be opened directly. 
            // Otherwise, the app list will appear, offering both the "JUST ONCE" and "ALWAYS" options.
            //
            // NOTE:
            // これがfalseの場合も、対応できるアプリがない場合にエラーが発生することはない。
            // ただ何も起こらないのでユーザー的にはあまり良くない。
            // trueの場合は空のapp chooserと「対応できるアプリがありません」のようなテキストが表示される。
            let use_app_chooser = true;

            // Decides whether to exclude this app from the app chooser.  
            // This is effective only if this app is configured to receive [`INTENT.ACTION_SEND_MULTIPLE`](https://developer.android.com/reference/android/content/Intent#ACTION_SEND_MULTIPLE) or [`INTENT.ACTION_SEND`](https://developer.android.com/reference/android/content/Intent#ACTION_SEND).    
            // If set to true, ***use_app_chooser*** must also be true and on Android 7 or later; 
            // otherwise, this setting will be ignored. 
            let exclude_self_from_app_chooser = true;

            // Noneの場合、共有するファイルに設定されているMIME typeを用いる
            let common_mime_type = None;

            let uris = uris.into_iter().collect::<Vec<_>>();

            self.0.api
                .run_mobile_plugin::<Res>("shareFiles", Req { uris, common_mime_type, use_app_chooser, exclude_self_from_app_chooser })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Show app chooser for sharing file with other apps.    
    /// This function returns immediately after requesting to open the app chooser, 
    /// without waiting for the app’s response. 
    /// 
    /// The available apps depend on the MIME type associated with the file.  
    /// This does not result in an error even if no available apps are found. 
    /// An empty app chooser is displayed.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI to share.  
    /// Must be **readable**.  
    /// URIs converted directly from a path, such as via [`FileUri::from_path`], can **not** be used.  
    /// 
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_SEND>
    pub fn share_file(
        &self, 
        uri: &FileUri,
    ) -> crate::Result<()> {
        
        self.share_files([uri])
    }

    /// Show app chooser for opening file with other apps.   
    /// This function returns immediately after requesting to open the app chooser, 
    /// without waiting for the app’s response. 
    /// 
    /// The available apps depend on the MIME type associated with the file.  
    /// This does not result in an error even if no available apps are found. 
    /// An empty app chooser is displayed.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI to view.  
    /// Must be **readable**.  
    /// URIs converted directly from a path, such as via [`FileUri::from_path`], can **not** be used.  
    /// 
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_VIEW>
    pub fn open_file(
        &self, 
        uri: &FileUri,
    ) -> crate::Result<()> {

        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri, mime_type: Option<&'a str>, use_app_chooser: bool, exclude_self_from_app_chooser: bool });
            impl_de!(struct Res;);

            let use_app_chooser = true;
            let exclude_self_from_app_chooser = true;
            let mime_type = None;
    
            self.0.api
                .run_mobile_plugin::<Res>("viewFile", Req { uri, mime_type, use_app_chooser, exclude_self_from_app_chooser })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Show app chooser for opening dir with other apps.   
    /// This function returns immediately after requesting to open the app chooser, 
    /// without waiting for the app’s response. 
    ///   
    /// This does not result in an error even if no available apps are found. 
    /// An empty app chooser is displayed.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target dir URI to view.  
    /// Must be **readable**.  
    /// URIs converted directly from a path, such as via [`FileUri::from_path`], can **not** be used.  
    /// 
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_VIEW>
    pub fn open_dir(
        &self, 
        uri: &FileUri,
    ) -> crate::Result<()> {

        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri, use_app_chooser: bool, exclude_self_from_app_chooser: bool });
            impl_de!(struct Res;);

            let use_app_chooser = true;
            let exclude_self_from_app_chooser = true;
    
            self.0.api
                .run_mobile_plugin::<Res>("viewDir", Req { uri, use_app_chooser, exclude_self_from_app_chooser })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Show app chooser for editing file with other apps.   
    /// This function returns immediately after requesting to open the app chooser, 
    /// without waiting for the app’s response. 
    /// 
    /// The available apps depend on the MIME type associated with the file.  
    /// This does not result in an error even if no available apps are found. 
    /// An empty app chooser is displayed.
    /// 
    /// # Note
    /// I think that this may be the least commonly used request for sending file to app.  
    /// Even if you want to open an image or video editing app, 
    /// [`FileSender::open_file`] allows you to choose from a wider range of apps in many cases.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI to view.  
    /// Must be **read-writeable**.  
    /// URIs converted directly from a path, such as via [`FileUri::from_path`], can **not** be used.  
    /// 
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_EDIT>
    pub fn edit_file(
        &self, 
        uri: &FileUri,
    ) -> crate::Result<()> {

        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri, mime_type: Option<&'a str>, use_app_chooser: bool, exclude_self_from_app_chooser: bool });
            impl_de!(struct Res;);

            let use_app_chooser = true;
            let exclude_self_from_app_chooser = true;
            let mime_type = None;
    
            self.0.api
                .run_mobile_plugin::<Res>("editFile", Req { uri, mime_type, use_app_chooser, exclude_self_from_app_chooser })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Determines whether the specified files can be used with [`FileSender::share_files`].  
    /// If no app is available to handle the files, this returns false. 
    /// 
    /// # Args
    /// - ***uris*** :  
    /// Target file URIs to share.  
    /// This all needs to be **readable**.  
    /// 
    /// # Support
    /// All Android version.
    #[deprecated = "Since Android 11, This does not function correctly due to android security."]
    pub fn can_share_files<'b>(
        &self, 
        uris: impl IntoIterator<Item = &'b FileUri>, 
    ) -> crate::Result<bool> {

        on_android!({
            impl_se!(struct Req<'a> { uris: Vec<&'a FileUri>, common_mime_type: Option<&'a str> });
            impl_de!(struct Res { value: bool });

            let common_mime_type = None;
            let uris = uris.into_iter().collect::<Vec<_>>();

            self.0.api
                .run_mobile_plugin::<Res>("canShareFiles", Req { uris, common_mime_type })
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }

    /// Determines whether the specified file can be used with [`FileSender::share_file`].  
    /// If no app is available to handle the file, this returns false. 
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    #[deprecated = "Since Android 11, This does not function correctly due to android security."]
    pub fn can_share_file(&self, uri: &FileUri) -> crate::Result<bool> {
        #[allow(deprecated)]
        self.can_share_files([uri])
    }

    /// Determines whether the specified file can be used with [`FileSender::open_file`].  
    /// If no app is available to handle the file, this returns false. 
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    #[deprecated = "Since Android 11, This does not function correctly due to android security."]
    pub fn can_open_file(&self, uri: &FileUri) -> crate::Result<bool> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri, mime_type: Option<&'a str> });
            impl_de!(struct Res { value: bool });

            let mime_type = None;

            self.0.api
                .run_mobile_plugin::<Res>("canViewFile", Req { uri, mime_type })
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }

    /// Determines whether the specified file can be used with [`FileSender::edit_file`].  
    /// If no app is available to handle the file, this returns false. 
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// Must be **read-writeable**.  
    /// 
    /// # Support
    /// All Android version.
    #[deprecated = "Since Android 11, This does not function correctly due to android security."]
    pub fn can_edit_file(&self, uri: &FileUri) -> crate::Result<bool> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri, mime_type: Option<&'a str> });
            impl_de!(struct Res { value: bool });

            let mime_type = None;

            self.0.api
                .run_mobile_plugin::<Res>("canEditFile", Req { uri, mime_type })
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }
}