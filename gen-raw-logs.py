try:
    from typing import Any, Dict, Union, Optional
except:
    pass
import hashlib
import time
import string
import boto3
import json
import random

# import ip_asset_mapping_pb2
# from ip_asset_mapping_pb2 import IpAssetMapping, IpAssetMappings

def rand_str(l):
    # type: (int) -> str
    return ''.join(random.choice(string.ascii_uppercase + string.digits)
                   for _ in range(l))


class IpAssetMappingGenerator(object):
    def __init__(self, asset_id=None, ip=None, timestamp=None):
        # type: (Optional[str], Optional[str], Optional[int]) -> None
        self.asset_id = asset_id or rand_str(20)
        self.ip = ip or ".".join(str(random.randint(0, 255)) for _ in range(4))
        self.timestamp = timestamp or int(time.time())

    def generate_mapping(self):
        # type: () -> IpAssetMapping
        mapping = IpAssetMapping()
        mapping.asset_id = self.asset_id
        mapping.ip = self.ip
        mapping.timestamp = self.timestamp
        return mapping

class ProcessLogGenerator(object):

    def __init__(self):
        # type: () -> None
        self.inner_time = time.time()


    def generate_raw_log(self):
        # type: () -> Dict[str, Union[str, int]]
        self.inner_time = random.randint(50, 150)
        return {
            'pid': random.randint(2, 150),
            'ppid': random.randint(2, 150),
            'name': rand_str(20),
            'asset_id': rand_str(20),
            'arguments': rand_str(20),
            'timestamp': self.inner_time,
            'sourcetype': "PROCESS_START",
        }


def generate_basic_process_logs():
    logs = []
    asset = "asset_zg"

    logs.append(
        {
            'pid': 1,
            'ppid': 0,
            'name': "init",
            'asset_id': asset,
            'arguments': rand_str(20),
            'timestamp': 150,
            'sourcetype': "PROCESS_START",
        }
    )

    logs.append(
        {
            'pid': 2,
            'ppid': 1,
            'name': "explorer",
            'asset_id': asset,
            'arguments': rand_str(20),
            'timestamp': 250,
            'sourcetype': "PROCESS_START",
        }
    )


    logs.append(
        {
            'pid': 3,
            'ppid': 2,
            'name': "chrome",
            'asset_id': asset,
            'arguments': rand_str(20),
            'timestamp': 350,
            'sourcetype': "PROCESS_START",
        }
    )

    # Chrome creates malicious.doc
    logs.append(
        {
            "creator_pid": 3,
            "creator_name": "chrome",
            "path": "/home/user/downloads/malicious.doc",
            "asset_id": asset,
            "timestamp": 450,
            "sourcetype": "FILE_CREATE"
        }
    )

    # Explorer executes word.exe
    logs.append(
        {
            'pid': 4,
            'ppid': 2,
            'name': "word",
            'exe': "/user/program_files/word.exe",
            'asset_id': asset,
            'arguments': rand_str(20),
            'timestamp': 500,
            'sourcetype': "PROCESS_START",
        }
    )
    # Word.exe reads malware.doc

    logs.append(
        {
            "reader_pid": 4,
            "reader_name": "word",
            "path": "/home/user/downloads/malicious.doc",
            "asset_id": asset,
            "timestamp": 550,
            "sourcetype": "FILE_READ"
        }
    )

    # Word.exe downloads and creates payload.exe

    logs.append(
        {
            "creator_pid": 4,
            "creator_name": "word",
            "path": "/home/user/local/payload.exe",
            "asset_id": asset,
            "timestamp": 600,
            "sourcetype": "FILE_CREATE"
        }
    )

    # Word.exe spawns payload.exe

    logs.append(
        {
            'pid': 5,
            'ppid': 4,
            'name': "payload",
            'exe': "/home/user/local/payload.exe",
            'asset_id': asset,
            'arguments': rand_str(20),
            'timestamp': 650,
            'sourcetype': "PROCESS_START",
        }
    )

    return logs


def main():
    now = int(time.time())

    time.sleep(1)
    proc_generator = ProcessLogGenerator()

    # raw_logs = [proc_generator.generate_raw_log() for _ in (range(0, 100))]
    raw_logs = generate_basic_process_logs()
    # mappings = [IpAssetMappingGenerator(asset_id=p['asset_id'], timestamp=now)
    #                 .generate_mapping() for p in raw_logs]

    # print(mappings)
    print(raw_logs)
    epoch = int(time.time())

    # mapping = IpAssetMappings()
    # mapping.mappings.extend(mappings)
    # mapping_body = mapping.SerializeToString()
    serialized_raw_logs = json.dumps(raw_logs)
    # #
    #
    s3 = boto3.client('s3')
    s3.put_object(
        Body=serialized_raw_logs,
        Bucket="grapl-raw-log-bucket",
        Key=str(epoch - (epoch % (24 * 60 * 60))) + "/PROCESS_START/" + str(epoch)
    )
    # s3 = boto3.client('s3')
    # res = s3.put_object(
    #     Body=mapping_body,
    #     Bucket="grapl-identity-mappings",
    #     Key=str(epoch - (epoch % (24 * 60 * 60))) + "/ip_asset_mappings/" +
    #         str(epoch)
    # )
    # print(res)


if __name__ == '__main__':
    main()

