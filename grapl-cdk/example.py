

class FileManager(object):

    def upload_file(self, file_contents, path):
        pass

    def retrieve_file(self, file_contents, path):
        pass


class LocalFileManager(FileManager):

    def __init__(self):
        self.root_directory = "/home/andrea/"

    def upload_file(self, file_contents, path):
        open(self.root_directory + path).write(file_contents)

    def retrieve_file(self, path):
        return open(self.root_directory + path).read()


class S3FilLocalFileManagereManager(FileManager):

    def __init__(self):
        self.s3_client = s3Client()

    def upload_file(self, file_contents, path):
        pass
        # upload to s3

    def retrieve_file(self, path):
        return open(self.root_directory + path).read()


def handles_thing(file_manager: FileManager):

    file = "pathblah"
    file_manager.upload_file("contents")


def main():
    file_manager = S3FileManager()
    handles_thing(file_manager)
