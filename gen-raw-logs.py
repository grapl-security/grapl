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
import zstd

def rand_str(l):
    # type: (int) -> str
    return ''.join(random.choice(string.ascii_uppercase + string.digits)
                   for _ in range(l))


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


asset_a = "asset_zzm"
asset_b = "asset2_10"
asset_a_ip = "172.23.4.10"
asset_b_ip = "172.26.7.13"

def generate_basic_process_logs():
    logs = []

    cnc_ip = "12.34.45.56"

    logs.append(
        {
            'pid': 1,
            'ppid': 0,
            'name': "init",
            'asset_id': asset_a,
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
            'asset_id': asset_a,
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
            'asset_id': asset_a,
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
            "asset_id": asset_a,
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
            'asset_id': asset_a,
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
            "asset_id": asset_a,
            "timestamp": 550,
            "sourcetype": "FILE_READ"
        }
    )

    # word.exe external ip connection

    logs.append(
        {
            "pid": 4,
            "protocol": "TCP",
            "src_port": 253,
            "dst_port": 443,
            "src_addr": asset_a_ip,
            "dst_addr": cnc_ip,
            "timestamp": 585,
            "sourcetype": "OUTBOUND_TCP",
        }
    )

    # Word.exe creates payload.exe

    logs.append(
        {
            "creator_pid": 4,
            "creator_name": "word",
            "path": "/home/user/local/payload.exe",
            "asset_id": asset_a,
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
            'asset_id': asset_a,
            'arguments': rand_str(20),
            'timestamp': 650,
            'sourcetype': "PROCESS_START",
        }
    )

    # Payload spawns SSH
    logs.append(
        {
            'pid': 6,
            'ppid': 5,
            'name': "ssh",
            'exe': "/bin/ssh",
            'asset_id': asset_a,
            'arguments': "username@"+asset_b_ip,
            'timestamp': 700,
            'sourcetype': "PROCESS_START",
        }
    )

    # SSH connects to asset2

    logs.append(
        {
            "pid": 6,
            "protocol": "TCP",
            "src_port": 149,
            "dst_port": 22,
            "src_addr": asset_a_ip,
            "dst_addr": asset_b_ip,
            "timestamp": 750,
            "sourcetype": "OUTBOUND_TCP",
        }
    )

    # sshd spawns on asset2

    logs.append(
        {
            'pid': 7,
            'ppid': 1,
            'name': "sshd",
            'exe': "/bin/sshd",
            'asset_id': asset_b,
            'arguments': rand_str(20),
            'timestamp': 450,
            'sourcetype': "PROCESS_START",
        }
    )

    # sshd receives connection from asset_a
    logs.append(
        {
            "pid": 7,
            "protocol": "TCP",
            "src_port": 22,
            "dst_port": 149,
            "src_addr": asset_b_ip,
            "dst_addr": asset_a_ip,
            "timestamp": 751,
            "sourcetype": "INBOUND_TCP",
        }
    )

    # sshd spawns child process bash
    logs.append(
        {
            'pid': 8,
            'ppid': 7,
            'name': "bash",
            'exe': "/bin/bash",
            'asset_id': asset_b,
            'arguments': rand_str(20),
            'timestamp': 850,
            'sourcetype': "PROCESS_START",
        }
    )

    return logs


def identity_mappings():
    return [
        {
            "ip": asset_b_ip,
            "asset_id": asset_b,
            "timestamp": 10,
        },
        {
            "ip": asset_a_ip,
            "asset_id": asset_a,
            "timestamp": 10,
        },
    ]


def main():
    now = int(time.time())

    time.sleep(1)
    proc_generator = ProcessLogGenerator()

    raw_logs = generate_basic_process_logs()
    print(raw_logs)
    epoch = int(time.time())

    mapping_body = zstd.compress(json.dumps(identity_mappings()), 4)
    serialized_raw_logs = zstd.compress(json.dumps(raw_logs), 4)

    s3 = boto3.client('s3')


    res = s3.put_object(
        Body=mapping_body,
        Bucket="grapl-identity-mappings-bucket",
        Key=str(epoch - (epoch % (24 * 60 * 60))) + "/ip_asset_mappings/" +
            str(epoch)
    )
    time.sleep(2)

    s3.put_object(
        Body=serialized_raw_logs,
        Bucket="grapl-raw-log-bucket",
        Key=str(epoch - (epoch % (24 * 60 * 60))) + "/PROCESS_START/" + str(epoch)
    )

    print(res)


if __name__ == '__main__':
    main()

