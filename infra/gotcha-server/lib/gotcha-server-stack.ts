import * as cdk from "aws-cdk-lib";
import * as apigateway from "aws-cdk-lib/aws-apigatewayv2";
import * as logs from "aws-cdk-lib/aws-logs";
import * as iam from "aws-cdk-lib/aws-iam";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as s3 from "aws-cdk-lib/aws-s3";
import * as s3deploy from "aws-cdk-lib/aws-s3-deployment";
import { Stage, StageProps } from "aws-cdk-lib";
import { Construct } from "constructs";

export class GotchaApplicationStage extends Stage {
  constructor(scope: Construct, id: string, props?: StageProps) {
    super(scope, id, props);

    new GotchaServerStack(this, "GotchaServerStack", props);
  }
}

class GotchaServerStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);
    const stageName = Stage.of(this)?.stageName.toLowerCase() ?? "oops";

    // TODO: Move cargo-lambda from CI to here
    const lambdaFunction = lambda.Function.fromFunctionArn(
      this,
      "GotchaServer",
      `arn:aws:lambda:${this.region}:${this.account}:function:gotcha-server-${stageName}`,
    );

    const api = new apigateway.CfnApi(this, "GotchaApi", {
      name: `gotcha-api-${stageName}`,
      protocolType: "HTTP",
      tags: {
        organization: "gotcha",
        environment: stageName,
      },
    });

    const integration = new apigateway.CfnIntegration(
      this,
      "LambdaIntegration",
      {
        apiId: api.ref,
        integrationType: "AWS_PROXY",
        integrationUri: lambdaFunction.functionArn,
        payloadFormatVersion: "2.0",
      },
    );

    new apigateway.CfnRoute(this, "DefaultRoute", {
      apiId: api.ref,
      routeKey: "$default",
      target: `integrations/${integration.ref}`,
    });

    const logGroup = new logs.LogGroup(this, "GotchaLogGroup", {
      logGroupName: `/aws/api-gateway/gotcha-api-${stageName}`,
      removalPolicy: cdk.RemovalPolicy.DESTROY,
    });

    new apigateway.CfnStage(this, "DefaultStage", {
      apiId: api.ref,
      stageName: "$default",
      autoDeploy: true,
      accessLogSettings: {
        destinationArn: logGroup.logGroupArn,
        format: JSON.stringify({
          requestId: "$context.requestId",
          status: "$context.status",
          integrationStatus: "$context.integrationStatus",
          requestTime: "$context.requestTime",
          requestTimeEpoch: "$context.requestTimeEpoch",
          path: "$context.path",
          errorMessage: "$context.error.message",
        }),
      },
      tags: {
        organization: "gotcha",
        environment: stageName,
      },
    });

    new lambda.CfnPermission(this, `ApiGatewayInvoke`, {
      action: "lambda:InvokeFunction",
      functionName: `gotcha-server-${stageName}`,
      principal: "apigateway.amazonaws.com",
      sourceArn: `arn:aws:execute-api:${this.region}:${this.account}:${api.ref}/*`,
    });

    logGroup.grantWrite(new iam.ServicePrincipal("apigateway.amazonaws.com"));

    // S3 bucket for static files
    const staticFilesBucket = new s3.Bucket(this, "StaticFilesBucket", {
      bucketName: `gotcha-public-${stageName}`,
      publicReadAccess: true,
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ACLS,
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      autoDeleteObjects: true,
    });

    // Deploy static files to S3 (assuming dist folder exists in project root)
    new s3deploy.BucketDeployment(this, "DeployStaticFiles", {
      sources: [s3deploy.Source.asset("../../dist")],
      destinationBucket: staticFilesBucket,
    });

    new cdk.CfnOutput(this, "ApiUrl", {
      value: `https://${api.ref}.execute-api.${this.region}.amazonaws.com/`,
    });

    new cdk.CfnOutput(this, "StaticFilesUrl", {
      value: `https://${staticFilesBucket.bucketName}.s3.${this.region}.amazonaws.com/`,
    });

    new cdk.CfnOutput(this, "S3BucketName", {
      value: staticFilesBucket.bucketName,
    });
  }
}
