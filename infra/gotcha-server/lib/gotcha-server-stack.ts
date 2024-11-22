import * as cdk from "aws-cdk-lib";
import * as apigateway from "aws-cdk-lib/aws-apigatewayv2";
import * as logs from "aws-cdk-lib/aws-logs";
import * as iam from "aws-cdk-lib/aws-iam";
import * as lambda from "aws-cdk-lib/aws-lambda";
import { Construct } from "constructs";

export class GotchaServerStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const api = new apigateway.CfnApi(this, "GotchaApi", {
      name: "gotcha-api",
      protocolType: "HTTP",
      tags: {
        organization: "gotcha",
      },
    });

    const integration = new apigateway.CfnIntegration(
      this,
      "LambdaIntegration",
      {
        apiId: api.ref,
        integrationType: "AWS_PROXY",
        integrationUri: `arn:aws:lambda:${this.region}:${this.account}:function:gotcha-server`,
        payloadFormatVersion: "2.0",
      },
    );

    new apigateway.CfnRoute(this, "DefaultRoute", {
      apiId: api.ref,
      routeKey: "$default",
      target: `integrations/${integration.ref}`,
    });

    const logGroup = new logs.LogGroup(this, "GotchaLogGroup", {
      logGroupName: "/aws/api-gateway/gotcha-api",
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
      },
    });

    const lambdaFunction = lambda.Function.fromFunctionArn(
      this,
      "GotchaServer",
      `arn:aws:lambda:${this.region}:${this.account}:function:gotcha-server`,
    );

    lambdaFunction.addPermission("ApiGatewayInvoke", {
      principal: new iam.ServicePrincipal("apigateway.amazonaws.com"),
      sourceArn: `arn:aws:execute-api:${this.region}:${this.account}:${api.ref}/*`,
    });

    logGroup.grantWrite(new iam.ServicePrincipal("apigateway.amazonaws.com"));

    new cdk.CfnOutput(this, "ApiUrl", {
      value: `https://${api.ref}.execute-api.${this.region}.amazonaws.com/`,
    });
  }
}
