import { Stack } from '@aws-cdk/core';
import { Watchful } from '../lib';
import { expect as cdk_expect, haveResource } from '@aws-cdk/assert';

test('creates an empty dashboard', () => {
  // GIVEN
  const stack = new Stack();

  // WHEN
  new Watchful(stack, 'watchful');

  // THEN
  cdk_expect(stack).to(haveResource('AWS::CloudWatch::Dashboard'));
});

