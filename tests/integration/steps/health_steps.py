import json
import urllib.request

from behave import when, then


@when('I check the facade health')
def step_check_health(context):
    resp = urllib.request.urlopen(f"{context.base_url}/health", timeout=5)
    context.health_response = json.loads(resp.read())


@then('the health status should be "{status}"')
def step_health_status(context, status):
    assert context.health_response["status"] == status, (
        f"expected {status}, got {context.health_response['status']}"
    )


@then('the health response should include service "{service}" with status "{status}"')
def step_health_service(context, service, status):
    services = context.health_response["services"]
    assert service in services, f"{service} not in health response"
    assert services[service]["status"] == status
