// Built-in templates for the "샘플" dropdown (SPEC UI-8).

export interface Sample {
  id: string;
  label: string;
  content: string;
}

const MINIMAL = `{
  "rules": [
    {
      "rule-type": "selection",
      "rule-id": "1",
      "rule-name": "include-all",
      "rule-action": "include",
      "object-locator": { "schema-name": "%", "table-name": "%" }
    }
  ]
}
`;

const FILTERED = `{
  "rules": [
    {
      "rule-type": "selection",
      "rule-id": "1",
      "rule-name": "include-hr",
      "rule-action": "include",
      "object-locator": { "schema-name": "HR", "table-name": "%" },
      "filters": [
        {
          "filter-type": "source",
          "column-name": "emp_id",
          "filter-conditions": [
            { "filter-operator": "gte", "value": "100" },
            { "filter-operator": "lte", "value": "9999" }
          ]
        }
      ]
    },
    {
      "rule-type": "transformation",
      "rule-id": "2",
      "rule-name": "rename-table",
      "rule-action": "rename",
      "rule-target": "table",
      "object-locator": { "schema-name": "HR", "table-name": "employees" },
      "value": "emp"
    }
  ]
}
`;

const WITH_ERRORS = `{
  "rules": [
    {
      "rule-type": "selection",
      "rule-id": "1",
      "rule-name": "explicit-with-wildcard",
      "rule-action": "explicit",
      "object-locator": { "schema-name": "HR", "table-name": "emp%" }
    },
    {
      "rule-type": "table-settings",
      "rule-id": "1",
      "rule-name": "ts-wildcard",
      "object-locator": { "schema-name": "HR", "table-name": "ord%" },
      "lob-settings": { "mode": "limited", "bulk-max-size": 0 }
    }
  ]
}
`;

export const SAMPLES: Sample[] = [
  { id: "minimal", label: "최소 selection", content: MINIMAL },
  { id: "filtered", label: "필터 + 변환", content: FILTERED },
  { id: "errors", label: "오류 예시 (학습용)", content: WITH_ERRORS },
];
