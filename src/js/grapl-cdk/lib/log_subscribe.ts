import * as cdk from '@aws-cdk/core';
import * as s3 from '@aws-cdk/aws-s3';
import * as sns from '@aws-cdk/aws-sns';
import {SubscriptionFilterOptions} from '@aws-cdk/aws-logs/lib/log-group
import * as s3n from '@aws-cdk/aws-s3-notifications';

import { RemovalPolicy } from '@aws-cdk/core';
import { BlockPublicAccess, BucketEncryption } from '@aws-cdk/aws-s3';
import {filterUndefined} from "@aws-cdk/core/lib/util";


export class LogSubscribe {
    //readonly filter: logs.SubscriptionFilter;
    readonly filterOpts: SubscriptionFilterOptions;

    constructor(scope: cdk.Construct, eventName: string) {

        this.filterOpts = {
            destination: undefined,
            filterPattern: {
                logPatternString: "MONITORING|"
            }
        }


    }
}
