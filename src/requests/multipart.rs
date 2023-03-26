/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use bytes::{BufMut, Bytes, BytesMut};

/// Represents the data of a `multipart/form-data` body.
pub struct Form {
    fields: Vec<(FieldKey, Bytes)>,
    boundary: String,
}

struct FieldKey {
    name: String,
    file_name: Option<String>,
}

impl Form {
    /// Creates a new form.
    pub fn new() -> Self {
        Form {
            fields: Vec::new(),
            // surely no one would include such a ridiculous string in a file
            boundary: "-sdsaddsfgdghhskjtretyetertertedh?-".to_string(),
        }
    }

    /// Gets the content type of this form, including the boundary.
    pub fn content_type(&self) -> String {
        format!(
            "multipart/form-data; boundary=\"{}\"",
            quote(&self.boundary)
        )
    }

    /// Adds a text field to this form.
    /// The field doesn't have a file name.
    pub fn text<K: AsRef<str>, V: AsRef<str>>(&mut self, field_name: K, text: V) {
        let bytes = Bytes::from(text.as_ref().to_string());
        let key = FieldKey {
            name: field_name.as_ref().to_string(),
            file_name: None,
        };
        self.fields.push((key, bytes));
    }

    /// Adds a named file to this form.
    pub fn file<K, F>(&mut self, field_name: K, file_name: F, data: Bytes)
    where
        K: AsRef<str>,
        F: AsRef<str>,
    {
        let key = FieldKey {
            name: field_name.as_ref().to_string(),
            file_name: Some(file_name.as_ref().to_string()),
        };
        self.fields.push((key, data));
    }

    /// Converts this form into its serialised `multipart/form-data` format.
    pub fn bytes(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        let dashes = b"--";
        let boundary = self.boundary.as_bytes();
        let crlf = b"\r\n";

        for (key, data) in &self.fields {
            bytes.put_slice(dashes);
            bytes.put_slice(boundary);
            bytes.put_slice(crlf);
            bytes.put_slice(b"Content-Disposition: form-data; name=\"");
            bytes.put_slice(quote(&key.name).as_bytes());
            bytes.put_slice(b"\"");

            if let Some(file_name) = &key.file_name {
                bytes.put_slice(b"; filename=\"");
                bytes.put_slice(quote(file_name).as_bytes());
                bytes.put_slice(b"\"");
            }

            bytes.put_slice(crlf);
            bytes.put_slice(crlf);
            bytes.put_slice(data);
            bytes.put_slice(crlf);
        }

        bytes.put_slice(dashes);
        bytes.put_slice(boundary);
        bytes.put_slice(dashes);

        bytes.freeze()
    }
}

/// Quotes the contents of a string according to RFC 822, 3.3: quoted-string.
fn quote(str: &String) -> String {
    let mut result = String::new();

    for c in str.chars() {
        // See RFC 822 3.3: quoted-string
        if c == '\\' || c == '\r' || c == '"' {
            result.push('\\');
        }

        result.push(c);
    }

    result
}
