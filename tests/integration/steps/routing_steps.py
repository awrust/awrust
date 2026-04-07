from behave import given, when, then
from botocore.exceptions import ClientError


@given('bucket "{bucket}" exists')
def step_bucket_exists(context, bucket):
    context.s3.create_bucket(Bucket=bucket)


@when('I create bucket "{bucket}"')
def step_create_bucket(context, bucket):
    context.s3.create_bucket(Bucket=bucket)


@then('bucket "{bucket}" should appear in the bucket list')
def step_bucket_in_list(context, bucket):
    names = [b["Name"] for b in context.s3.list_buckets()["Buckets"]]
    assert bucket in names, f"{bucket} not in {names}"


@then('bucket "{bucket}" should exist')
def step_bucket_should_exist(context, bucket):
    context.s3.head_bucket(Bucket=bucket)


@given('object "{path}" contains "{content}"')
def step_object_contains(context, path, content):
    bucket, key = path.split("/", 1)
    context.s3.put_object(Bucket=bucket, Key=key, Body=content.encode())


@when('I put object "{path}" with content "{content}"')
def step_put_object(context, path, content):
    bucket, key = path.split("/", 1)
    context.s3.put_object(Bucket=bucket, Key=key, Body=content.encode())


@then('object "{path}" should contain "{content}"')
def step_object_should_contain(context, path, content):
    bucket, key = path.split("/", 1)
    body = context.s3.get_object(Bucket=bucket, Key=key)["Body"].read()
    assert body == content.encode(), f"expected {content!r}, got {body!r}"


@then('object "{path}" should exist')
def step_object_should_exist(context, path):
    bucket, key = path.split("/", 1)
    context.s3.head_object(Bucket=bucket, Key=key)


@then('object "{path}" should not exist')
def step_object_should_not_exist(context, path):
    bucket, key = path.split("/", 1)
    try:
        context.s3.head_object(Bucket=bucket, Key=key)
        assert False, f"{path} still exists"
    except ClientError as e:
        assert e.response["Error"]["Code"] == "404"


@when('I upload "{path}" as multipart with {n:d} parts')
def step_multipart_upload(context, path, n):
    bucket, key = path.split("/", 1)
    mpu = context.s3.create_multipart_upload(Bucket=bucket, Key=key)
    upload_id = mpu["UploadId"]

    parts = []
    for i in range(1, n + 1):
        data = bytes([i]) * (5 * 1024 * 1024)
        resp = context.s3.upload_part(
            Bucket=bucket, Key=key, UploadId=upload_id,
            PartNumber=i, Body=data,
        )
        parts.append({"PartNumber": i, "ETag": resp["ETag"]})

    context.s3.complete_multipart_upload(
        Bucket=bucket, Key=key, UploadId=upload_id,
        MultipartUpload={"Parts": parts},
    )


@when('I copy "{src}" to "{dst}"')
def step_copy_object(context, src, dst):
    dst_bucket, dst_key = dst.split("/", 1)
    context.s3.copy_object(Bucket=dst_bucket, CopySource=src, Key=dst_key)


@when('I batch delete "{keys}" from "{bucket}"')
def step_batch_delete(context, keys, bucket):
    objects = [{"Key": k.strip()} for k in keys.split(",")]
    context.s3.delete_objects(Bucket=bucket, Delete={"Objects": objects})
