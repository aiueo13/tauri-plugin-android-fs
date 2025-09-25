# Version 15.0.1
- Update documentation.

# Version 15.0.0
- Remove `PublicStorage::app_dir_name`
- Remove arg `use_app_dir` from
    `PublicStorage::create_new_file`,
    `PublicStorage::create_dir_all`,
    `PublicStorage::resolve_path`,

- Update documentation.

# Version 14.0.0
- Change args of `WritableStream::sync_data`
- Change args of `WritableStream::sync_all`
- Remove `PublicStorage::get_available_volumes`
- Remove `PublicStorageVolume`
- Remove `PublicStorageVolumeId`
- Add `StorageVolume`
- Add `StorageVolumeId`
- Add `OutsidePrivateDir`
- Add `PublicStorage::get_volumes`
- Add `PrivateStorage::resolve_outside_path`
- Add `PrivateStorage::get_primary_volume`
- Add `PrivateStorage::get_volumes`
- Update documentation.

# Version 13.1.0
- Add `PrivateDir::NoBackupData`
- Update documentation.

# Version 13.0.0
- Deprecate and change args of `PrivateStorage::resolve_uri`
- Remove `AndroidFs::create_file`
- Remove `InitialLocation`
- Remove `PublicStorage::create_file`
- Remove `PublicStorage::create_file_in_app_dir`
- Remove `PublicStorage::create_dia_all_in_app_dir`
- Remove `PrivateStorage::resolve_path_with`
- Remove `PrivateStorage::resolve_uri_with`
- Remove wrapper functions of std::fs in `PrivateStorage`
- Add `VisualMediaTarget::ImageOrVideo { mime_type }`
- Add `AndroidFs::create_new_file`
- Add `PublicStorage::create_new_file`
- Add `PublicStorage::resolve_path`
- Add `PublicStorage::resolve_initial_location_top`
- Change args of `PublicStorage::create_dia_all`
- Change args of `PublicStorage::resolve_initial_location`
- Update documentation.

# Version 12.0.1
- Update documentation.

# Version 12.0.0
- Remove deperecated items.
- Deprecate `AndroidFs::resolve_initial_location`
- Change `InitialLocation`
- Add `api_level`
- Add `PublicStorageVolume`
- Add `PublicStorageVolumeId`
- Add `PublicStorage::get_volumes`
- Add `PublicStorage::get_primary_volume`
- Add `PublicStorage::resolve_initial_location`
- Add arg of `volume` to `PublicStorage::create_file`, PublicStorage::create_file_in_app_dir`, `PublicStorage::create_dir_all`, PublicStorage::create_dir_all_in_app_dir`
- Update documentation.

# Version 11.0.1
- Update documentation.

# Version 11.0.0
- Remove feature of `avoid-issue1`
- Remove deperecated items.
- Deprecate `AndroidFs::write_via_kotlin_in`
- Deprecate `AndroidFs::resolve_uri`
- Change returned type and behavior of `AndroidFs::get_mime_type`
- Remove arg of `multiple` from `FilePicker::pick_files`
- Remove arg of `multiple` from `FilePicker::pick_visual_medias`
- Remove arg of `multiple` from `FilePicker::pick_contents`
- Add arg of `buffer_size` to `AndroidFs::copy_via_kotlin`
- Add `AndroidFs::get_type`
- Add `AndroidFs::open_file_writable`
- Add `AndroidFs::open_file_readable`
- Add `AndroidFs::open_writable_stream`
- Add `AndroidFs::open_writable_stream_via_kotlin`
- Add `AndroidFs::api_level`
- Add `WritableStream`
- Add `EntryType`
- Update documentation.

# Version 10.2.3
- Update documentation.

# Version 10.2.2
- Update documentation.

# Version 10.2.1
- Update documentation.

# Version 10.2.0
- Deprecate `AndroidFs::can_share_file` 
- Deprecate `AndroidFs::can_view_file` 
- Deprecate `AndroidFs::show_share_file_dialog`
- Deprecate `AndroidFs::show_view_file_dialog`
- Deprecate `AndroidFs::show_open_file_dialog`
- Deprecate `AndroidFs::show_open_visual_media_dialog`
- Deprecate `AndroidFs::show_open_content_dialog`
- Deprecate `AndroidFs::show_save_file_dialog`
- Deprecate `AndroidFs::show_manage_dir_dialog`
- Add `AndroidFs::file_sender`
- Add `AndroidFs::file_picker`
- Add `FileSender`
- Add `FilePicker`
- Update documentation.

# Version 10.1.0
- Fix an issue that directory creation by `AndroidFs::resolve_initial_locaiton` may not working properly
- Add `InitialLocation::DirInPublicAppDir`
- Update documentation.

# Version 10.0.1
- Update documentation.

# Version 10.0.0
- Remove deperecated items.
- Change `Error` from enum to stcuct.
- Add `AndroidFs::create_dir_all`
- Add `PublicStorage::create_dir_all`
- Add `PublicStorage::create_dir_all_in_app_dir`
- Update documentation.

# Version 9.5.0
- Add `AndroidFs::rename`
- Add `AndroidFs::copy`
- Add `AndroidFs::try_resolve_file_uri`
- Add `AndroidFs::try_resolve_dir_uri`
- Add `PublicStorage::create_file`
- Add `PublicStorage::create_file_in_app_dir`
- Add `PublicStorage::app_dir_name`
- Expose fields of `FileUri`
- Deprecate `PublicStorage::create_file_in_public_dir`
- Deprecate `PublicStorage::create_file_in_public_app_dir`
- Synchronise folder creation on Kotolin side
- Update documentation.

# Version 9.4.0
- Add `AndroidFs::resolve_uri`
- Improvement `AndroidFs::resolve_initial_location`.
- Update documentation.

# Version 9.3.0
- Add `ImageFormat`
- Deprecate `DecodeOption`

# Version 9.2.1
- Fix an issue where video thumbnails may not be retrieved in `AndroidFs::get_thumbnail_to` and `AndroidFs::get_thumbnail`

# Version 9.2.0
- Add `AndroidFs::get_thumbnail`
- Add `AndroidFs::get_thumbnail_to`
- Fix an issue where the UI would freeze and become ANR error if the response was too long when using `AndroidFs::open_file`.
- Update documentation.

# Version 9.1.0
- Add `AndroidFs::remove_dir_all`
- Fix a bug that allowed non-files to be deleted with `AndroidFs::remove_file`.
- Fix a bug that allowed non-empty-dir to be deleted with `AndroidFs::remove_dir`.
- Update documentation.

# Version 9.0.0
- Change the API provider, such as `AndroidFs`, `PublicStorage`, and `PrivateStorage`, from trait to struct.
- Update documentation.

# Version 8.4.0
- Add `AndroidFs::resolve_initial_location`
- Update documentation.

# Version 8.3.2
- Fix a bug that caused document generation to fail in doc.rs.

# Version 8.3.1
- Add `AndroidFs::show_open_content_dialog`
- Update documentation.

# Version 8.3.0
- Add `AndroidFs::show_open_content_dialog`
- Update documentation.

# Version 8.2.1
- Update documentation.

# Version 8.2.0
- Change behaviour when `None` is specified for mime_type, of `AndroidFs::create_file`, `AndroidFs::show_save_file_dialog` `PublicStorage::create_file_in_public_dir`, and `PublicStorage::create_file_in_public_app_dir`.
- Update documentation.

# Version 8.1.1
- Update documentation.

# Version 8.1.0
- Add `AndroidFs::show_share_file_dialog`
- Add `AndroidFs::show_view_file_dialog`
- Add `AndroidFs::can_share_file`
- Add `AndroidFs::can_view_file`
- Update documentation.

# Version 8.0.0
- Remove arg of mode from `AndroidFs::take_persistable_uri_permission`
- Add `AndroidFs::copy_via_kotlin`
- Add `AndroidFs::write_via_kotlin`
- Add `AndroidFs::write_via_kotlin_in`
- Add `AndroidFs::need_write_via_kotlin`
- Change the `AndroidFs::show_save_file_dialog` to return no None even when a file on Google Drive is selected.
- Update documentation.

# Version 7.0.2
- Update documentation.

# Version 7.0.1
- Update documentation.

# Version 7.0.0
- Add `AndroidFs::check_persisted_uri_permission`
- Add `PersistableAccessMode::Read`
- Add `PersistableAccessMode::Write`
- Deprecate `PersistableAccessMode::ReadOnly`
- Deprecate `PersistableAccessMode::WriteOnly`
- Deprecate `FileAccessMode::Write`
- Update documentation.

# Version 6.0.0
- Add `AndroidFs::take_persistable_uri_permission`
- Add `AndroidFs::release_persisted_uri_permission`
- Add `AndroidFs::release_all_persisted_uri_permissions`
- Add `AndroidFs::get_all_persisted_uri_permissions`
- Add `PersistedUriPermission`
- Remove arg `take_persistable_uri_permission` from  
    `AndroidFs::show_open_file_dialog`, 
    `AndroidFs::show_open_visual_media_dialog`, 
    `AndroidFs::show_save_file_dialog`
- Update documentation.

# Version 5.0.1
- Update documentation.

# Version 5.0.0
- Remove deperecated items.
- Remove `PersistableAccessMode`
- Remove `AndroidFs::take_persistable_uri_permission`
- Change `Entry { byte_size, .. }` to `Entry { len, .. }`
- Add `#[non_exhaustive]` attributes to `VisualMediaTarget`
- Add arg `take_persistable_uri_permission` to 
    `AndroidFs::show_open_file_dialog`, 
    `AndroidFs::show_open_visual_media_dialog`, 
    `AndroidFs::show_save_file_dialog`
- Add arg `initial_location` to 
    `AndroidFs::show_open_file_dialog`, 
    `AndroidFs::show_save_file_dialog`
- Add `AndroidFs::show_manage_dir_dialog`
- Deprecate `AndroidFs::show_open_dir_dialog`
- Update documentation.

# Version 4.5.3
- Change the `AndroidFs::show_save_file_dialog` to return None when a file on Google Drive is selected.
- Update documentation.

# Version 4.5.2
- Fix an issue where the UI would freeze and become ANR error if the response was too long when using `AndroidFs::read_dir`.

# Version 4.5.1
- Fix documentation.

# Version 4.5.0
- Add `FileUri::to_string`
- Add `FileUri::from_str`

# Version 4.4.2
- Update documentation.

# Version 4.4.1
- Update documentation.

# Version 4.4.0
- Update documentation.
- Add `PrivateStorage::create_new_file`

# Version 4.3.0
- Update documentation.
- Add `PublicStorage`
- Add `AndroidFs::public_storage`
- Deprecate `AndroidFs::create_file_in_public_location`
- Deprecate `AndroidFs::is_public_audiobooks_dir_available`
- Deprecate `AndroidFs::is_public_recordings_dir_available`

# Version 4.2.0
- Update documentation.
- Add feature `avoid-issue1`
- Add permission `allow-noop`
- Deprecate `PathError`
- Deprecate `Error::Path`

# Version 4.1.2
- Update documentation.

# Version 4.1.1
- Update documentation.

# Version 4.1.0
- Add `FileAccessMode::WriteTruncate`
- Update documentation.

# Version 4.0.0
Overall changes.

# Version 3.0.1
- Update documentation.

# Version 3.0.0
- Add `Entry`
- Remove `EntryPath`
- Change `AndroidFs::read_dir`
- Change `AndroidFs::get_mime_type`
- Update documentation.

# Version 2.0.1
- Improve performance of `AndroidFs::get_dir_name`
- Update documentation.

# Version 2.0.0
- Fix issue where `AndroidFs::get_dir_name` isn’t returning the correct directory name.
- Fix issue where `AndroidFs::read_dir` isn’t returning the correct subdirectory path.
- Fix issue where `AndroidFs::new_file` isn’t creating the file in the correct location.
- Add `#[non_exhaustive]` attributes to some enums
- Remove some deprecated functions.
- Change `convert_string_to_dir_path` and return-value format.
- Change `convert_dir_path_to_string` and return-value format.
- Change `AndroidFs::read_dir`.
- Change `Error::PluginInvoke(anyhow::Error)` to `Error::PluginInvoke(String)`
- Update documentation.

# ~~Version 1.6.0~~ (***Yanked***)
- Add `AndroidFs::remove_file`.
- Add `AndroidFs::new_file`.
- Add `AndroidFs::new_file_with_contents`.
- Update documentation.

# ~~Version 1.5.0~~ (***Yanked***)
- Deprecate `AndroidFs::take_persistable_read_permission`.
- Deprecate `AndroidFs::take_persistable_write_permission`.
- Add `DirPath`.
- Add `EntryPath`.
- Add `PersistableAccessMode`.
- Add `convert_string_to_dir_path`.
- Add `convert_dir_path_to_string`.
- Add `AndroidFs::show_open_dir_dialog`.
- Add `AndroidFs::read_dir`.
- Add `AndroidFs::get_dir_name`.
- Add `AndroidFs::grant_persistable_file_access`.
- Add `AndroidFs::grant_persistable_dir_access`.
- Update documentation.
- Remove files that should not be included.

# Version 1.4.2
- Remove files that should not be included.

# Version 1.4.1
- Update documentation.

# Version 1.4.0
- Add `AndroidFs::take_persistable_read_permission`.
- Add `AndroidFs::take_persistable_write_permission`.
- Add `convert_string_to_file_path`.
- Add `convert_file_path_to_string`.
- Update documentation.

# Version 1.3.2
- Update documentation.

# Version 1.3.1
- Update documentation.

# Version 1.3.0
- Deprecate `AndroidFs::open_file_writable`.
- Add `AndroidFs::create_file`.
- Add `PrivateStorage::create_file`.
- Update documentation.

# Version 1.2.0
- Add `AndroidFs::get_mime_type`.
- Update documentation.
