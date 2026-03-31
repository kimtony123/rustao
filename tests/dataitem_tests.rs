#[cfg(test)]
mod tests {
    use rustao::dataitem::DataItem;
    use rustao::schema::Tag;

    #[test]
    fn test_dataitem_new() {
        let di = DataItem::new("target".into(), b"data".to_vec(), vec![Tag::new("k", "v")], b"anchor".to_vec());
        assert_eq!(di.target, "target");
        assert_eq!(di.data, b"data");
        assert_eq!(di.tags[0].name, "k");
        assert_eq!(di.anchor, b"anchor");
    }

    #[test]
    fn test_anchor_truncation() {
        let long_anchor = vec![0u8; 40];
        let di = DataItem::new("".into(), vec![], vec![], long_anchor);
        assert_eq!(di.anchor.len(), 32);
    }

    #[test]
    fn test_get_signature_data() {
        let mut di = DataItem::new("target".into(), b"data".to_vec(), vec![Tag::new("k", "v")], b"anchor".to_vec());
        di.owner = b"owner".to_vec();
        let sig_data = di.get_signature_data().unwrap();
        assert!(!sig_data.is_empty());
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let original = DataItem::new(
            "target".into(),
            b"data".to_vec(),
            vec![Tag::new("k", "v")],
            b"anchor".to_vec(),
        );
        let mut with_dummy = original.clone();
        with_dummy.owner = b"owner".to_vec();
        with_dummy.signature = b"signature".to_vec();
        with_dummy.id = with_dummy.compute_id().unwrap();

        let serialized = with_dummy.serialize().unwrap();
        let deserialized = DataItem::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.target, with_dummy.target);
        assert_eq!(deserialized.data, with_dummy.data);
        assert_eq!(deserialized.tags, with_dummy.tags);
        assert_eq!(deserialized.anchor, with_dummy.anchor);
        assert_eq!(deserialized.owner, with_dummy.owner);
        assert_eq!(deserialized.signature, with_dummy.signature);
        assert_eq!(deserialized.id, with_dummy.id);
    }

    #[test]
    fn test_compute_id() {
        let di = DataItem::new("target".into(), b"data".to_vec(), vec![], b"".to_vec());
        let id = di.compute_id().unwrap();
        assert!(!id.is_empty());
        assert!(!id.contains('='));
    }
}