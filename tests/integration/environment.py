import os
import socket
import subprocess
import time
import urllib.request

import boto3


def _free_port():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def _wait_for_health(url, timeout=30):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            urllib.request.urlopen(f"{url}/health", timeout=1)
            return
        except Exception:
            time.sleep(0.2)
    raise RuntimeError(f"facade did not become ready at {url}")


def _start_cargo(context, port):
    root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

    subprocess.run(
        ["cargo", "build", "-p", "awrust"],
        cwd=root,
        check=True,
        capture_output=True,
    )

    env = os.environ.copy()
    env["AWRUST_LISTEN_ADDR"] = f"127.0.0.1:{port}"
    env["AWRUST_SERVICES"] = "s3"
    env["AWRUST_LOG"] = "warn"
    env["AWRUST_S3_STORE"] = "memory"

    context.server = subprocess.Popen(
        ["cargo", "run", "-p", "awrust"],
        cwd=root,
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def _start_docker(context, port, image):
    cmd = [
        "docker", "run", "--rm", "-d",
        "-p", f"{port}:4566",
        "--name", f"awrust-bdd-{port}",
        "-e", "AWRUST_SERVICES=s3",
        "-e", "AWRUST_LOG=warn",
        image,
    ]

    subprocess.run(cmd, capture_output=True, text=True, check=True)
    context._container = f"awrust-bdd-{port}"


def before_all(context):
    endpoint = os.environ.get("AWRUST_ENDPOINT")

    if endpoint:
        context.base_url = endpoint
    else:
        port = _free_port()
        context.port = port
        context.base_url = f"http://127.0.0.1:{port}"

        image = os.environ.get("IMAGE")

        if image:
            _start_docker(context, port, image)
        else:
            _start_cargo(context, port)

    _wait_for_health(context.base_url)

    context.s3 = boto3.client(
        "s3",
        endpoint_url=context.base_url,
        aws_access_key_id="test",
        aws_secret_access_key="test",
        region_name="us-east-1",
    )


def after_all(context):
    if hasattr(context, "_container"):
        subprocess.run(
            ["docker", "rm", "-f", context._container],
            capture_output=True,
            timeout=10,
        )
    if hasattr(context, "server"):
        context.server.terminate()
        context.server.wait(timeout=5)
