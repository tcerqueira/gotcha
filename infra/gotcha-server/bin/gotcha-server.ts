#!/usr/bin/env node
import * as cdk from "aws-cdk-lib";
import { GotchaServerStack } from "../lib/gotcha-server-stack";

const app = new cdk.App();
new GotchaServerStack(app, "GotchaServerStack", {
  env: { account: "597088030785", region: process.env.AWS_DEFAULT_REGION },
});
