"""Pytest fixtures shared across the test suite."""

from __future__ import annotations

import pytest

from apps.users.tests.factories import UserFactory


@pytest.fixture
def user(db):
    return UserFactory()


@pytest.fixture
def staff_user(db):
    return UserFactory(is_staff=True)


@pytest.fixture
def authed_client(client, user):
    client.force_login(user)
    return client
