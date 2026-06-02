#!/usr/bin/env python3
"""
Validate that the contract specification matches the actual implementation.

This script:
1. Parses the contract source code to extract actual function signatures
2. Compares against the spec to ensure completeness and accuracy
3. Reports discrepancies and missing functions/events

Usage:
    python3 scripts/validate-spec.py
    
Exit codes:
    0: Spec is valid and complete
    1: Spec validation failed (missing or incorrect items)
"""

import json
import re
from pathlib import Path
from typing import Set, Dict, List, Tuple
import sys


class ContractValidator:
    """Validate contract spec against actual implementation."""
    
    def __init__(self, contract_path: str, spec_path: str):
        self.contract_path = Path(contract_path)
        self.spec_path = Path(spec_path)
        
        if not self.contract_path.exists():
            raise FileNotFoundError(f"Contract file not found: {contract_path}")
        if not self.spec_path.exists():
            raise FileNotFoundError(f"Spec file not found: {spec_path}")
        
        self.source = self.contract_path.read_text()
        with open(self.spec_path) as f:
            self.spec = json.load(f)
        
        self.errors = []
        self.warnings = []
        self.info = []
    
    def validate(self) -> bool:
        """Run all validation checks."""
        print("=" * 70)
        print("NeuroWealth Vault - Contract Specification Validator")
        print("=" * 70)
        
        self._validate_functions()
        self._validate_events()
        self._validate_spec_structure()
        
        return self._report_results()
    
    def _validate_functions(self):
        """Check that all functions in spec exist in contract."""
        print("\n📋 Validating Functions...")
        
        # Extract function names from contract
        contract_functions = self._extract_contract_functions()
        spec_functions = {f["name"] for f in self.spec.get("functions", [])}
        
        print(f"   Contract has {len(contract_functions)} functions")
        print(f"   Spec documents {len(spec_functions)} functions")
        
        # Check for missing functions in spec
        missing_in_spec = contract_functions - spec_functions
        if missing_in_spec:
            for func in sorted(missing_in_spec):
                self.errors.append(f"Function '{func}' in contract but not in spec")
        
        # Check for extra functions in spec
        extra_in_spec = spec_functions - contract_functions
        if extra_in_spec:
            for func in sorted(extra_in_spec):
                self.warnings.append(f"Function '{func}' in spec but not in contract (may be deprecated)")
        
        # Validate function parameters
        self._validate_function_parameters(contract_functions)
        
        if not missing_in_spec and not extra_in_spec:
            self.info.append("✓ All contract functions are documented in spec")
    
    def _extract_contract_functions(self) -> Set[str]:
        """Extract all public function names from contract."""
        # Match "pub fn name(" or "pub async fn name("
        pattern = r'pub\s+(?:async\s+)?fn\s+(\w+)\s*\('
        matches = re.findall(pattern, self.source)
        return set(matches)
    
    def _validate_function_parameters(self, contract_functions: Set[str]):
        """Validate that function parameters match spec."""
        for func_name in contract_functions:
            # Find function in contract
            pattern = rf'pub\s+(?:async\s+)?fn\s+{func_name}\s*\(([^)]*)\)'
            match = re.search(pattern, self.source)
            if not match:
                continue
            
            params = match.group(1)
            
            # Find function in spec
            spec_func = next((f for f in self.spec.get("functions", []) 
                             if f["name"] == func_name), None)
            if not spec_func:
                continue
            
            # Check if 'env' parameter is documented
            if "env" in params and not any(p.get("name") == "env" 
                                          for p in spec_func.get("parameters", [])):
                self.warnings.append(f"Function '{func_name}': 'env' parameter not documented in spec")
    
    def _validate_events(self):
        """Check that all events are documented."""
        print("\n📢 Validating Events...")
        
        # Extract event types from contract
        event_pattern = r'pub struct (\w*Event)\s*\{'
        contract_events = set(re.findall(event_pattern, self.source))
        spec_events = {e["name"] for e in self.spec.get("events", [])}
        
        print(f"   Contract defines {len(contract_events)} events")
        print(f"   Spec documents {len(spec_events)} events")
        
        # Check for missing events
        missing_events = contract_events - spec_events
        if missing_events:
            for event in sorted(missing_events):
                self.warnings.append(f"Event '{event}' defined in contract but not documented in spec")
        
        # Check for extra events
        extra_events = spec_events - contract_events
        if extra_events:
            for event in sorted(extra_events):
                self.warnings.append(f"Event '{event}' in spec but not found in contract")
        
        if not missing_events and not extra_events:
            self.info.append("✓ All contract events are documented in spec")
    
    def _validate_spec_structure(self):
        """Validate that spec has required structure."""
        print("\n🔍 Validating Spec Structure...")
        
        required_fields = ["version", "contract", "network", "functions", "events", "errors"]
        missing_fields = [f for f in required_fields if f not in self.spec]
        
        if missing_fields:
            for field in missing_fields:
                self.errors.append(f"Spec missing required field: '{field}'")
        else:
            self.info.append("✓ Spec has all required top-level fields")
        
        # Validate function structure
        for func in self.spec.get("functions", []):
            if "name" not in func:
                self.errors.append("Function spec missing 'name' field")
            if "description" not in func:
                self.warnings.append(f"Function '{func.get('name')}' missing 'description'")
            if "parameters" not in func:
                self.warnings.append(f"Function '{func.get('name')}' missing 'parameters'")
        
        # Validate event structure
        for event in self.spec.get("events", []):
            if "name" not in event:
                self.errors.append("Event spec missing 'name' field")
            if "description" not in event:
                self.warnings.append(f"Event '{event.get('name')}' missing 'description'")
    
    def _report_results(self) -> bool:
        """Report validation results."""
        print("\n" + "=" * 70)
        print("Validation Results")
        print("=" * 70)
        
        if self.info:
            print("\n✅ Passed Checks:")
            for msg in self.info:
                print(f"   {msg}")
        
        if self.warnings:
            print("\n⚠️  Warnings:")
            for msg in self.warnings:
                print(f"   ⚠️  {msg}")
        
        if self.errors:
            print("\n❌ Errors:")
            for msg in self.errors:
                print(f"   ❌ {msg}")
            return False
        
        print("\n✅ All validation checks passed!")
        return True


def main():
    """Main entry point."""
    contract_path = "neurowealth-vault/contracts/vault/src/lib.rs"
    spec_path = "contract-spec.json"
    
    try:
        validator = ContractValidator(contract_path, spec_path)
        success = validator.validate()
        exit(0 if success else 1)
        
    except FileNotFoundError as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        exit(1)
    except json.JSONDecodeError as e:
        print(f"❌ Invalid JSON in spec: {e}", file=sys.stderr)
        exit(1)
    except Exception as e:
        print(f"❌ Validation failed: {e}", file=sys.stderr)
        exit(1)


if __name__ == "__main__":
    main()
