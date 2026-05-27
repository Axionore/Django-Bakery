"""Tenant admin — list of every tenant + their primary domain.

Staff-only. Tenant creation through the admin is fine for early-stage; once
you have self-service signup, replace with a signed-up-tenant onboarding flow
in `apps/users/views.py` that calls the same `create_tenant` mgmt command
under the hood.
"""

from __future__ import annotations

from django.contrib import admin
from django_tenants.admin import TenantAdminMixin

from apps.tenants.models import Domain, Tenant


@admin.register(Tenant)
class TenantAdmin(TenantAdminMixin, admin.ModelAdmin):
    list_display = ("name", "schema_name", "paid_until", "on_trial", "created_on")
    search_fields = ("name", "schema_name")
    readonly_fields = ("schema_name", "created_on")


@admin.register(Domain)
class DomainAdmin(admin.ModelAdmin):
    list_display = ("domain", "tenant", "is_primary")
    list_filter = ("is_primary",)
    search_fields = ("domain",)
