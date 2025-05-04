pub const TMP_DIR_RELATIVE_PATH: &str = "pluginAndroidFs-tmpDir-33bd1538-4434-dc4e-7e2f-515405cccbf9";

pub fn encode_document_id(input: &str) -> String {
    // https://developer.android.com/reference/android/net/Uri.html#encode(java.lang.String)
    
    const SAFE: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
        .remove(b'_')
        .remove(b'-')
        .remove(b'!')
        .remove(b'.')
        .remove(b'~')
        .remove(b'\'')
        .remove(b'(')
        .remove(b')')
        .remove(b'*');

    percent_encoding::utf8_percent_encode(input, SAFE).to_string()
}