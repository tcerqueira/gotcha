#!/usr/bin/env node
import * as cdk from "aws-cdk-lib";
import { GotchaApplicationStage } from "../lib/gotcha-server-stack";

const app = new cdk.App();
new GotchaApplicationStage(app, "Prod", {
  env: {
    account: "597088030785",
    region: process.env.AWS_DEFAULT_REGION,
  },
});

new GotchaApplicationStage(app, "Preview", {
  env: {
    account: "597088030785",
    region: process.env.AWS_DEFAULT_REGION,
  },
});
