import json
import boto3

cwlogs = boto3.client('logs')

loglist = cwlogs.describe_log_groups(
    logGroupNamePrefix='engagement'
)

#writes json output to file...
with open('loglist.json', 'w') as outfile:
    json.dump(loglist, outfile, ensure_ascii=False, indent=4,
              sort_keys=True)

#Opens file and searches through to find given loggroup name
with open("loglist.json") as f:
    file_parsed = json.load(f)

for i in file_parsed['logGroups']:
    print(i['logGroupName'])
    cwlogs.delete_log_group(
        logGroupName=(i['logGroupName'])
    )

