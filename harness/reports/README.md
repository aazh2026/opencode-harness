# Reports Module

## Purpose

Stores verification reports documenting test execution results. Reports capture comparison outcomes, verification verdicts, and identified gaps.

## Directory Structure

```
reports/
└── <suite>/
    └── <timestamp>.json  # Report files
```

## Key Types

- **Report**: Complete verification report
- **TestResult**: Individual test outcome
- **GapEntry**: Identified mismatch or contract gap

## Usage

Reports are generated after verification runs and stored for historical analysis. Each report contains:
- Execution metadata
- Test results (pass/fail/skipped)
- Identified gaps
- Failure classifications

## Path Convention

Reports follow: `harness/reports/<suite>/<timestamp>.json`
