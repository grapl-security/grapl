import * as s3 from '@aws-cdk/aws-s3';
import { Construct } from '@aws-cdk/core';

export class GraplS3Bucket extends s3.Bucket {
    /*
    Automatically enables features like:
    - autodeleting objects (simplifying `cdk destroy`)
    */
    constructor(scope: Construct, id: string, props?: s3.BucketProps) {
        super(scope, id, {
            ...props,
            autoDeleteObjects: true,
        })
    }
}