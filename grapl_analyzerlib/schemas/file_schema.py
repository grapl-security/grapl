from grapl_analyzerlib.schemas.schema_builder import NodeSchema


class FileSchema(NodeSchema):
    def __init__(self):
        super(FileSchema, self).__init__()
        (
            self.with_str_prop("file_name")
            .with_str_prop("asset_id")
            .with_str_prop("file_path")
            .with_str_prop("file_extension")
            .with_str_prop("file_mime_type")
            .with_int_prop("file_size")
            .with_str_prop("file_version")
            .with_str_prop("file_description")
            .with_str_prop("file_product")
            .with_str_prop("file_company")
            .with_str_prop("file_directory")
            .with_int_prop("file_inode")
            .with_str_prop("file_hard_links")
            .with_bool_prop("signed")
            .with_str_prop("signed_status")
            .with_str_prop("md5_hash")
            .with_str_prop("sha1_hash")
            .with_str_prop("sha256_hash")
        )

    @staticmethod
    def self_type() -> str:
        return "File"
