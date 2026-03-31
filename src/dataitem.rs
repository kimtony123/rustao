use crate::error::{Error, Result};
use crate::schema::Tag;
use crate::signer::ARSigner;
use crate::utils::{base64url_encode, read_varint, sha256, write_varint};

#[derive(Debug, Clone)]
pub struct DataItem {
    pub target: String,
    pub data: Vec<u8>,
    pub tags: Vec<Tag>,
    pub anchor: Vec<u8>,
    pub owner: Vec<u8>,
    pub signature: Vec<u8>,
    pub id: String,
}

impl DataItem {
    pub fn new(target: String, data: Vec<u8>, tags: Vec<Tag>, anchor: Vec<u8>) -> Self {
        let anchor = if anchor.len() > 32 { anchor[..32].to_vec() } else { anchor };
        Self {
            target,
            data,
            tags,
            anchor,
            owner: Vec::new(),
            signature: Vec::new(),
            id: String::new(),
        }
    }

    pub fn sign(&mut self, signer: &ARSigner) -> Result<()> {
        let data_to_sign = self.get_signature_data()?;
        self.signature = signer.sign(&data_to_sign)?;
        self.owner = signer.public_key().to_vec();
        self.id = self.compute_id()?;
        Ok(())
    }

    pub fn get_signature_data(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        // Owner
        write_varint(&mut buf, self.owner.len() as u64);
        buf.extend_from_slice(&self.owner);

        // Target
        let target_bytes = self.target.as_bytes();
        write_varint(&mut buf, target_bytes.len() as u64);
        buf.extend_from_slice(target_bytes);

        // Anchor
        write_varint(&mut buf, self.anchor.len() as u64);
        buf.extend_from_slice(&self.anchor);

        // Tags
        write_varint(&mut buf, self.tags.len() as u64);
        for tag in &self.tags {
            let name_bytes = tag.name.as_bytes();
            write_varint(&mut buf, name_bytes.len() as u64);
            buf.extend_from_slice(name_bytes);

            let value_bytes = tag.value.as_bytes();
            write_varint(&mut buf, value_bytes.len() as u64);
            buf.extend_from_slice(value_bytes);
        }

        // Data
        write_varint(&mut buf, self.data.len() as u64);
        buf.extend_from_slice(&self.data);

        Ok(buf)
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut buf = self.get_signature_data()?;
        write_varint(&mut buf, self.signature.len() as u64);
        buf.extend_from_slice(&self.signature);
        Ok(buf)
    }

    pub fn compute_id(&self) -> Result<String> {
        let data_to_sign = self.get_signature_data()?;
        let hash = sha256(&data_to_sign);
        Ok(base64url_encode(&hash))
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        let mut pos = 0;

        // Owner
        let (owner_len, new_pos) = read_varint(data, pos).map_err(|e| Error::DataItem(e))?;
        pos = new_pos;
        if pos + owner_len as usize > data.len() {
            return Err(Error::DataItem("unexpected end of data reading owner".into()));
        }
        let owner = data[pos..pos + owner_len as usize].to_vec();
        pos += owner_len as usize;

        // Target
        let (target_len, new_pos) = read_varint(data, pos).map_err(|e| Error::DataItem(e))?;
        pos = new_pos;
        let target = if target_len > 0 {
            let target_bytes = &data[pos..pos + target_len as usize];
            pos += target_len as usize;
            String::from_utf8(target_bytes.to_vec()).map_err(|_| Error::DataItem("invalid target UTF-8".into()))?
        } else {
            String::new()
        };

        // Anchor
        let (anchor_len, new_pos) = read_varint(data, pos).map_err(|e| Error::DataItem(e))?;
        pos = new_pos;
        let anchor = data[pos..pos + anchor_len as usize].to_vec();
        pos += anchor_len as usize;

        // Tags
        let (tag_count, new_pos) = read_varint(data, pos).map_err(|e| Error::DataItem(e))?;
        pos = new_pos;
        let mut tags = Vec::with_capacity(tag_count as usize);
        for _ in 0..tag_count {
            let (name_len, new_pos) = read_varint(data, pos).map_err(|e| Error::DataItem(e))?;
            pos = new_pos;
            let name_bytes = &data[pos..pos + name_len as usize];
            pos += name_len as usize;
            let name = String::from_utf8(name_bytes.to_vec()).map_err(|_| Error::DataItem("invalid tag name UTF-8".into()))?;

            let (value_len, new_pos) = read_varint(data, pos).map_err(|e| Error::DataItem(e))?;
            pos = new_pos;
            let value_bytes = &data[pos..pos + value_len as usize];
            pos += value_len as usize;
            let value = String::from_utf8(value_bytes.to_vec()).map_err(|_| Error::DataItem("invalid tag value UTF-8".into()))?;

            tags.push(Tag::new(&name, &value));
        }

        // Data
        let (data_len, new_pos) = read_varint(data, pos).map_err(|e| Error::DataItem(e))?;
        pos = new_pos;
        let payload = data[pos..pos + data_len as usize].to_vec();
        pos += data_len as usize;

        // Signature
        let (sig_len, new_pos) = read_varint(data, pos).map_err(|e| Error::DataItem(e))?;
        pos = new_pos;
        let signature = data[pos..pos + sig_len as usize].to_vec();

        let mut di = Self::new(target, payload, tags, anchor);
        di.owner = owner;
        di.signature = signature;
        di.id = di.compute_id()?;
        Ok(di)
    }
}