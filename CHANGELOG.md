vNext (Month Day, Year)
-----------------------
- (When complete) Add profile file provider for region (#594, #xyz)

v0.20 (August 10th, 2021)
--------------------------

**Breaking changes**

- (#635) The `config()`, `config_mut()`, `request()`, and `request_mut()` methods on `operation::Request` have been renamed to `properties()`, `properties_mut()`, `http()`, and `http_mut()` respectively.
- (#635) The `Response` type on Tower middleware has been changed from `http::Response<SdkBody>` to `operation::Response`. The HTTP response is still available from the `operation::Response` using its `http()` and `http_mut()` methods.
- (#635) The `ParseHttpResponse` trait's `parse_unloaded()` method now takes an `operation::Response` rather than an `http::Response<SdkBody>`.
- (#626) `ParseHttpResponse` no longer has a generic argument for the body type, but instead, always uses `SdkBody`. This may cause compilation failures for you if you are using Smithy generated types to parse JSON or XML without using a client to request data from a service. The fix should be as simple as removing `<SdkBody>` in the example below:

  Before:
  ```rust
  let output = <Query as ParseHttpResponse<SdkBody>>::parse_loaded(&parser, &response).unwrap();
  ```

  After:
  ```rust
  let output = <Query as ParseHttpResponse>::parse_loaded(&parser, &response).unwrap();
  ```

**New This Week**
- Add AssumeRoleProvider parser implementation. (#632)
- The closure passed to `async_provide_credentials_fn` can now borrow values (#637)
- Add `Sender`/`Receiver` implementations for Event Stream (#639)
- Bring in the latest AWS models (#630)

v0.19 (August 3rd, 2021)
------------------------

IoT Data Plane is now available! If you discover it isn't functioning as expected, please let us know!

This week also sees the addition of a robust async caching credentials provider.
Take a look at the
[STS example](https://github.com/awslabs/smithy-rs/blob/7fa4af4a9367aeca6d55e26fc4d4ba93093b90c4/aws/sdk/examples/sts/src/bin/credentials-provider.rs)
to see how to use it.

**New This Week**

- :tada: Add IoT Data Plane (#624)
- :tada: Add LazyCachingCredentialsProvider to aws-auth for use with expiring credentials, such as STS AssumeRole. Update STS example to use this new provider (#578, #595)
- :bug: Correctly encode HTTP Checksums using base64 instead of hex. Fixes aws-sdk-rust#164. (#615)
- Update SDK gradle build logic to use gradle properties (#620)
- Overhaul serialization/deserialization of numeric/boolean types. This resolves issues around serialization of NaN/Infinity and should also reduce the number of allocations required during serialization. (#618)
- Update SQS example to clarify usage of FIFO vs. standard queues (#622, @trevorrobertsjr)
- Implement Event Stream frame encoding/decoding (#609, #619)

**Contributions**

Thank you for your contributions! :heart:

- @trevorrobertsjr (#622)


v0.18.1 (July 27th 2021)
------------------------

- Remove timestreamwrite and timestreamquery from the generated services (#613)


v0.18 (July 27th 2021)
----------------------

**Breaking changes**

- `test-util` has been made an optional dependency and has moved from
  aws-hyper to smithy-http. If you were relying on `aws_hyper::TestConnection`, add `smithy-client` as a dependency
  and enable the optional `test-util` feature. This prunes some unnecessary dependencies on `roxmltree` and `serde_json`
  for most users. (#608)

**New This Week**

- :tada: Release all but three remaining AWS services! Glacier, IoT Data Plane and Transcribe streaming will be available in a future release. If you discover that a service isn't functioning as expected please let us know! (#607)
- :bug: Bugfix: Fix parsing bug where parsing XML incorrectly stripped whitespace (#590, aws-sdk-rust#153)
- Establish common abstraction for environment variables (#594)
- Add windows to the test matrix (#594)
- :bug: Bugfix: Constrain RFC-3339 timestamp formatting to microsecond precision (#596)


v0.17 (July 15th 2021)
----------------------

**New this Week**

- :tada: Add support for Autoscaling (#576, #582)
- `AsyncProvideCredentials` now introduces an additional lifetime parameter, simplifying bridging it with `#[async_trait]` interfaces
- Fix S3 bug when content type was set explicitly (aws-sdk-rust#131, #566, @eagletmt)

**Contributions**

Thank you for your contributions! :heart:

- @eagletmt (#566)


v0.16 (July 6th 2021)
---------------------

**New this Week**

- :warning: **Breaking Change:** `ProvideCredentials` and `CredentialError` were both moved into `aws_auth::provider` when they were previously in `aws_auth` (#572)
- :tada: Add support for AWS Config (#570)
- :tada: Add support for EBS (#567)
- :tada: Add support for Cognito (#573)
- :tada: Add support for Snowball (#579, @landonxjames)
- Make it possible to asynchronously provide credentials with `async_provide_credentials_fn` (#572, #577)
- Improve RDS, QLDB, Polly, and KMS examples (#561, #560, #558, #556, #550)
- Update AWS SDK models (#575)
- :bug: Bugfix: Fill in message from error response even when it doesn't match the modeled case format (#565)

**Internal Changes**

- Add support for `@unsignedPayload` Smithy trait (#567)
- Strip service/api/client suffix from sdkId (#546)
- Remove idempotency token trait (#571)

**Contributions**

Thank you for your contributions! :heart:

- landonxjames (#579)


v0.15 (June 29th 2021)
----------------------

This week, we've added EKS, ECR and Cloudwatch. The JSON deserialization implementation has been replaced, please be
on the lookout for potential issues.

**New this Week**

- :tada: Add support for ECR (#557)
- :tada: Add support for Cloudwatch (#554)
- :tada: Add support for EKS (#553)
- :warn: **Breaking Change:** httpLabel no longer causes fields to be non-optional. (#537)
- :warn: **Breaking Change:** `Exception` is not renamed to `Error`. Code may need to be updated to replace `exception` with `error`
- Add more SES examples, and improve examples for Batch.
- Improved error handling ergonomics: Errors now provide `is_<variantname>()` methods to simplify error handling
- :bug: Bugfix: fix bug where invalid query strings could be generated (#531, @eagletmt)

**Internal Changes**
- Pin CI version to 1.52.1 (#532)
- New JSON deserializer implementation (#530)
- Fix numerous namespace collision bugs (#539)
- Gracefully handle empty response bodies during JSON parsing (#553)

**Contributors**

Thank you for your contributions! :heart:

- @eagletmt (#531)


v0.14 (June 22nd 2021)
----------------------

This week, we've added CloudWatch Logs support and fixed several bugs in the generated S3 clients.
There are a few breaking changes this week.

**New this Week**

- :tada: Add support for CloudWatch Logs (#526)
- :warning: **Breaking Change:** The `set_*` functions on generated Builders now always take an `Option` (#506)
- :warning: **Breaking Change:** Unions with Documents will see the inner document type change from `Option<Document>` to `Document` (#520)
- :warning: **Breaking Change:** The `as_*` functions on unions now return `Result` rather than `Option` to clearly indicate what the actual value is (#527)
- Add more S3 examples, and improve SNS, SQS, and SageMaker examples. Improve example doc comments (#490, #508, #509, #510, #511, #512, #513, #524)
- :bug: Bugfix: Show response body in trace logs for calls that don't return a stream (#514)
- :bug: Bugfix: Correctly parse S3's GetBucketLocation response (#516)
- :bug: Bugfix: Correctly URL-encode tilde characters before SigV4 signing (#519)
- :bug: Bugfix: Fix S3 PutBucketLifecycle operation by adding support for the `@httpChecksumRequired` Smithy trait (#523)
- :bug: Bugfix: Correctly parse non-list headers with commas in them (#525, @eagletmt)

**Internal Changes**
- Reduce name collisions in generated code (#502)
- Combine individual example packages into per-service example packages with multiple binaries (#481, #490)
- Re-export HyperAdapter in smithy-client (#515, @zekisherif)
- Add serialization/deserialization benchmark for DynamoDB to exercise restJson1 generated code (#507)

**Contributions**

Thank you for your contributions! :heart:

- @eagletmt (#525)
- @zekisherif (#515)

v0.13 (June 15th 2021)
----------------------

Smithy-rs now has codegen support for all AWS services! This week, we've added CloudFormation, SageMaker, EC2, and SES. More details below.

**New this Week**

- :tada: Add support for CloudFormation (#500, @alistaim)
- :tada: Add support for SageMaker (#473, @alistaim)
- :tada: Add support for EC2 (#495)
- :tada: Add support for SES (#499)
- Add support for the EC2 Query protocol (#475)
- Generate fluent builders for all smithy-rs clients (#496, @jonhoo)
- :bug: Bugfix: RFC-3339 timestamps (`date-time` format in Smithy) are now formatted correctly (#479, #489)
- :bug: Bugfix: Union and enum variants named Self no longer cause compile errors in generated code (#492)

**Internal Changes**

- Combine individual example packages into per-service example packages with multiple binaries (#477, #480, #482, #484, #485, #486, #487, #491)
- Work towards JSON deserialization overhaul (#474)
- Make deserializer function naming consistent between XML and JSON deserializers (#497)

Contributors:
- @Doug-AWS
- @jdisanti
- @rcoh
- @alistaim
- @jonhoo

Thanks!!

v0.12 (June 8th 2021)
---------------------

Starting this week, smithy-rs now has codegen support for all AWS services except EC2. This week we’ve added MediaLive, MediaPackage, SNS, Batch, STS, RDS, RDSData, Route53, and IAM. More details below.

**New this Week**

- :tada: Add support for MediaLive and MediaPackage (#449, @alastaim)
- :tada: Add support for SNS (#450)
- :tada: Add support for Batch (#452, @alistaim)
- :tada: Add support for STS. **Note:** This does not include support for an STS-based credential provider although an example is provided. (#453)
- :tada: Add support for RDS (#455) and RDS-Data (#470). (@LMJW)
- :tada: Add support for Route53 (#457, @alistaim)
- Support AWS Endpoints & Regions. With this update, regions like `iam-fips` and `cn-north-1` will now resolve to the correct endpoint. Please report any issues with endpoint resolution. (#468)
- :bug: Bugfix: Primitive numerics and booleans are now filtered from serialization when they are 0 and not marked as required. This resolves issues where maxResults needed to be set even though it is optional. (#451)
- :bug: Bugfix: S3 Head Object returned the wrong error when the object did not exist (#460, fixes #456)

**Internal Changes**

- Remove unused key “build” from smithy-build.json and Rust settings (#447)
- Split SDK CI jobs for faster builds & reporting (#446)
- Fix broken doc link in JSON serializer (@LMJW)
- Work towards JSON deserialization overhaul (#454, #462)

Contributors:
- @rcoh
- @jdisanti
- @alistaim
- @LMJW

Thanks!!

v0.11 (June 1st, 2021)
----------------------

**New this week:**

- :tada: Add support for SQS. SQS is our first service to use the awsQuery protocol. Please report any issues you may encounter.
- :tada: Add support for ECS.
- **Breaking Change**: Refactored `smithy_types::Error` to be more flexible. Internal fields of `Error` are now private and can now be accessed accessor functions. (#426)
- `ByteStream::from_path` now accepts `implications AsRef<Path>` (@LMJW)
- Add support for S3 extended request id (#429)
- Add support for the awsQuery protocol. smithy-rs can now add support for all services except EC2.
- **Bugfix**: Timestamps that fell precisely on minute boundaries were not properly formatted (#435)
- Improve documentation for `ByteStream` & add `pub use` (#443)
- Add support for `EndpointPrefix` used by [`s3::WriteGetObjectResponse`](https://awslabs.github.io/aws-sdk-rust/aws_sdk_s3/operation/struct.WriteGetObjectResponse.html) (#420)

**Smithy Internals**

- Rewrite JSON serializer (#411, #423, #416, #427)
- Remove dead “rootProject” setting in `smithy-build.json`
- **Bugfix:** Idempotency tokens were not properly generated when operations were used by resources

Contributors:
- @jdisanti
- @rcoh
- @LMJW

Thanks!
