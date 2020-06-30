import * as AWS from 'aws-sdk';
const fsPromises = require('fs').promises
import AdmZip from 'adm-zip';

// Set the region
// Create SQS service object
const sqs = new AWS.SQS();
const s3 = new AWS.S3();
// Replace with your accountid and the queue name you setup
const accountId = '131275825514';
const queueName = 'test';
const queueUrl = `https://sqs.us-east-1.amazonaws.com/${accountId}/${queueName}`;
// Setup the receiveMessage parameters
const params = {
    QueueUrl: queueUrl,
    MaxNumberOfMessages: 1,
    WaitTimeSeconds: 10,
};

const main = async (): Promise<void> => {
    const {Messages} = await sqs.receiveMessage(params).promise();

    if (!Messages) {
        return;
    }

    const msg = JSON.parse(Messages[0].Body || "{}");

    const record = msg["Records"][0];

    const bucket = record["s3"]["bucket"]["name"];
    const key = record["s3"]["object"]["key"];
    const s3Params = {
        Bucket: bucket,
        Key: key,
    };

    let packaged_payload = await s3.getObject(s3Params).promise();
    await fsPromises.writeFile('./package.zip', packaged_payload);

    const zip = new AdmZip("./my_file.zip");
    zip.extractAllTo(/*target path*/"./zips/", /*overwrite*/true);

}

(async () => {
    for (; ;) {
        await main();
    }
})().catch(e => {
    // Deal with the fact the chain failed
    console.error(e);
});

