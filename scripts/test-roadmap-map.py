#!/usr/bin/env python3
"""Regression tests for roadmap migration and prerequisite validation."""

import copy
import unittest

from check_roadmap_map import validate

PLAN = """### v0.55.0 - Published
Status: tagged.
### v0.56.0 - Kernel
Status: planned.
Goal: implement the kernel.

Scope: implementation. Depends on v0.55.0.

Deliverables:
- Kernel.
### v0.57.0 - Feature Completion
Status: planned.
Goal: integrate the feature.

Scope: completion. Depends on v0.56.0.

Deliverables:
- Integration.
"""

MANIFEST = {
    "source_commit": "a" * 40,
    "published_through": "v0.55.0",
    "retained_contracts": 1,
    "added_passes": 1,
    "mapping": [{"previous": "v0.56.0", "title": "Feature", "passes": ["v0.56.0", "v0.57.0"]}],
}


class MapTests(unittest.TestCase):
    def test_valid(self):
        self.assertEqual(validate(PLAN, MANIFEST), 2)

    def test_missing_or_wrong_scope(self):
        for change in (
            PLAN.replace("Scope:", "Boundary:", 1),
            PLAN.replace("Depends on v0.55.0", "Depends on v0.56.0"),
            PLAN.replace("Depends on v0.55.0", "Depends on v0.57.0"),
            PLAN.replace("Depends on v0.55.0", "Depends on v0.54.0"),
        ):
            with self.subTest(change=change), self.assertRaises(ValueError):
                validate(change, MANIFEST)

    def test_missing_or_renamed_completion(self):
        for change in (PLAN.replace("v0.57.0 -", "v0.58.0 -"),
                       PLAN.replace("Feature Completion", "Unknown Completion")):
            with self.subTest(change=change), self.assertRaises(ValueError):
                validate(change, MANIFEST)

    def test_duplicate_mapping(self):
        manifest = copy.deepcopy(MANIFEST)
        manifest["mapping"][0]["passes"] = ["v0.56.0", "v0.56.0"]
        with self.assertRaises(ValueError):
            validate(PLAN, manifest)

    def test_counts_and_published_baseline(self):
        for field, value in (("retained_contracts", 2), ("added_passes", 0),
                             ("published_through", "v0.54.0"), ("source_commit", "main")):
            manifest = copy.deepcopy(MANIFEST)
            manifest[field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                validate(PLAN, manifest)


if __name__ == "__main__":
    unittest.main()
