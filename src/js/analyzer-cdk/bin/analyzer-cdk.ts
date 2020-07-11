#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from '@aws-cdk/core';
import { AnalyzerCdkStack } from '../lib/analyzer-cdk-stack';

const app = new cdk.App();
new AnalyzerCdkStack(app, 'AnalyzerCdkStack');
