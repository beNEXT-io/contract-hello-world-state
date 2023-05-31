# contract-hello-world-state

This project contains source code and supporting files for an Accord Project Smart Legal Contract app developed as a serverless application. 

This contract app can be deployed a with the SAM CLI to an AWS account and is fully functional.

During the deployment process the following AWS infrastructure is created:

- an API Gateway (used for receiving requests and returning responses.)
- a Lambda function (which is a compute container that runs the Contract App's code logic).
- a DynamoDB Table (used to store Contract State).

## Requirements

* This template was tested with Rust v1.69.0 (84c898d65 2023-04-16).

* If this your first time installing the AWS SAM CLI then you'll need to setup an account on Amazon Web Services. This includes:

  - Signing up for an AWS account.

  - Creating an administrative IAM user.

  - Creating an access key ID and secret access key.

  - Installing the AWS CLI.

  - Configuring AWS credentials.

Please visit [AWS SAM prerequisites](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/prerequisites.html) to continue.

## Deploy the sample application

To deploy the application, you need the folllowing tools:

* SAM CLI - [Install the SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html)
* Docker (optional - for local testing only) - [Install Docker community edition](https://hub.docker.com/search/?type=edition&offering=community)
* [Rust](https://www.rust-lang.org/) version 1.64.0 or newer
* [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda) for cross-compilation

To build and deploy your application for the first time, run the following in your shell:

```bash
sam build
sam deploy
```

The first command will build the source of your application. The second command will package and deploy your application to AWS with the default `samconfig.toml` in the project. Alternatively, you can run `sam deploy --guided` to deploy with a series of prompts:

* **Stack Name**: The name of the stack to deploy to CloudFormation. This should be unique to your account and region, and a good starting point would be something matching your project name.
* **AWS Region**: The AWS region you want to deploy your app to.
* **Confirm changes before deploy**: If set to yes, any change sets will be shown to you before execution for manual review. If set to no, the AWS SAM CLI will automatically deploy application changes.
* **Allow SAM CLI IAM role creation**: Many AWS SAM templates, including this example, create AWS IAM roles required for the AWS Lambda function(s) included to access AWS services. By default, these are scoped down to minimum required permissions. To deploy an AWS CloudFormation stack which creates or modifies IAM roles, the `CAPABILITY_IAM` value for `capabilities` must be provided. If permission isn't provided through this prompt, to deploy this example you must explicitly pass `--capabilities CAPABILITY_IAM` to the `sam deploy` command.
* **Save arguments to `samconfig.toml`**: If set to yes, your choices will be saved to a configuration file inside the project, so that in the future you can just re-run `sam deploy` without parameters to deploy changes to your application.

## Test the deployed contract app.

You can test your deployed app by sending a request to the Contract's API Gateway Endpoint URL, which you can find in the output values displayed after deployment.

### 1. HelloWorldStateClause 

Used to populate the contract with the contract data. Receives back a copy of the stored data.

```
curl --request POST \
  --url https://{your-api-name}.execute-api.ap-southeast-2.amazonaws.com/Prod/{your-contract-id}/ \
  --header 'Content-Type: application/json' \
  --data '{
    "request": {
        "HelloWorldStateClause": {
  				"$class": "org.accordproject.helloworldstate.HelloWorldStateClause",
  				"name": "Fred Bloggs",
  				"clauseId": "8d16efc9-96af-458e-b7f2-e3367403d37e",
  				"$identifier": "8d16efc9-96af-458e-b7f2-e3367403d37e"
			}
    }
}'
```

**Example Response**
```
{
	"HelloWorldStateClause": {
		"$class": "org.accordproject.helloworldstate.HelloWorldStateClause",
		"name": "Fred Bloggs",
		"clauseId": "8d16efc9-96af-458e-b7f2-e3367403d37e",
		"$identifier": "8d16efc9-96af-458e-b7f2-e3367403d37e"
	}
}
```

### 2. MyRequest

Sends a request to the contract and receives a response based on the contract logic.

```
curl --request POST \
  --url https://{your-api-name}.execute-api.ap-southeast-2.amazonaws.com/Prod/{your-contract-id}/ \
  --header 'Content-Type: application/json' \
  --data '{
    "request": {
        "MyRequest": {
            "$class": "org.accordproject.helloworldstate.MyRequest",
            "input": "Accord Project",
            "$timestamp": "2023-05-24T14:56:45.123+0000"
        }
    }
}'
```

**Example Response**
```
{
	"MyResponse": {
		"$class": "org.accordproject.helloworldstate.MyRequest",
		"output": "Hello Jack Walnut - Accord Project",
		"$timestamp": "2023-05-29T13:40:22.522341554+00:00"
	}
}
```

## Fetch, tail, and filter Lambda function logs

To simplify troubleshooting, SAM CLI has a command called `sam logs`. `sam logs` lets you fetch logs generated by your deployed Lambda function from the command line. In addition to printing the logs on the terminal, this command has several nifty features to help you quickly find the bug.

`NOTE`: This command works for all AWS Lambda functions; not just the ones you deploy using SAM.

```bash
$ sam logs -n HelloWorldStateFunction --stack-name contract-hello-world-state --tail
```

You can find more information and examples about filtering Lambda function logs in the [SAM CLI Documentation](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-logging.html).

## Tests

Tests are defined alongside your lambda function code in the `rust_app/src` folder.

```bash
cargo test
```


## Cleanup

To delete the sample application that you created, use the AWS CLI. Assuming you used your project name for the stack name, you can run the following:

```bash
sam delete
```
