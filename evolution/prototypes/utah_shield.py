import os
import re
import json
import sqlite3
import time
from typing import Dict, List, Any, Tuple

class UtahShieldEngine:
    """
    High-Performance Resource Filtering and Security Shield Core.
    Interceptors map addresses against verified local rule matrices.
    """
    def __init__(self, db_path: str = "utah_shield_metrics.db"):
        self.db_path = db_path
        self.blocked_domains_cache: Dict[str, bool] = {}
        self.cosmetic_rules: List[str] = []
        self._initialize_database()
        self._load_baseline_rules()

    def _initialize_database(self) -> None:
        """Sets up the local ledger to store blocking statistics."""
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.cursor()
            # Create a table to track block metrics
            cursor.execute("""
                CREATE TABLE IF NOT EXISTS shield_logs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp REAL,
                    target_url TEXT,
                    rule_matched TEXT,
                    category TEXT
                )
            """)
            conn.commit()

    def _load_baseline_rules(self) -> None:
        """Loads default high-risk rules into the fast memory cache."""
        # Simulated standard ad networks and aggressive popup domains
        raw_rules = [
            ("doubleclick.net", "Advertising"),
            ("adservice.google.com", "Advertising"),
            ("popads.net", "Aggressive Popup"),
            ("trackingscript.xyz", "Tracker"),
            ("maliciousredirect.su", "Malware Risk"),
            ("fakesecurityalert.cc", "Exploit Attempt")
        ]
        for domain, category in raw_rules:
            self.blocked_domains_cache[domain] = True
            
        # Standard structural layout rules to neutralize hidden floating elements
        self.cosmetic_rules = [
            r"div\[class\*='ad-overlay'\]",
            r"iframe\[src\*='banner'\]",
            r"div\[id\^='popup-wrapper'\]"
        ]

    def inspect_request(self, current_page_url: str, resource_url: str) -> Tuple[bool, str]:
        """
        Evaluates a network request against the secure cache matrix.
        Returns a tuple indicating if the request should be blocked and its category.
        """
        # Extract the target host name from the resource web address
        host_match = re.search(r"https?://([^/\s]+)", resource_url)
        if not host_match:
            return False, "Allowed"

        target_host = host_match.group(1)
        
        # Check parent domains recursively to catch subdomains
        parts = target_host.split('.')
        for i in range(len(parts) - 1):
            check_domain = ".".join(parts[i:])
            if check_domain in self.blocked_domains_cache:
                self._log_blocked_event(resource_url, check_domain, "Network Filter")
                return True, "Blocked Network Request"

        return False, "Allowed"

    def process_page_layout(self, raw_html: str) -> Tuple[str, int]:
        """
        Scans page code to neutralize annoying floating segments and fake alerts.
        Returns the safe modified code and the number of elements neutralized.
        """
        modified_html = raw_html
        neutralized_count = 0
        
        # Neutralize malicious inline popup script patterns
        script_patterns = [
            r"window\.open\(.*?\)",
            r"eval\(atob\(.*?\)\)"
        ]
        
        for pattern in script_patterns:
            matches = re.findall(pattern, modified_html)
            if matches:
                neutralized_count += len(matches)
                modified_html = re.sub(pattern, "/* Neutralized Shield Element */", modified_html)

        return modified_html, neutralized_count

    def _log_blocked_event(self, url: str, rule: str, category: str) -> None:
        """Records a blocked item into the database file for UI display."""
        try:
            with sqlite3.connect(self.db_path) as conn:
                cursor = conn.cursor()
                cursor.execute(
                    "INSERT INTO shield_logs (timestamp, target_url, rule_matched, category) VALUES (?, ?, ?, ?)",
                    (time.time(), url, rule, category)
                )
                conn.commit()
        except sqlite3.Error:
            # Sane fallback to prevent browser disruption if disk is busy
            pass

    def get_dashboard_metrics(self) -> Dict[str, Any]:
        """Retrieves total protection counts grouped by category for the display screen."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.cursor()
            cursor.execute("SELECT COUNT(*) as total FROM shield_logs")
            total_row = cursor.fetchone()
            total_blocks = total_row["total"] if total_row else 0

            cursor.execute("SELECT category, COUNT(*) as count FROM shield_logs GROUP BY category")
            rows = cursor.fetchall()
            
            category_breakdown = {row["category"]: row["count"] for row in rows}
            
        return {
            "total_threats_prevented": total_blocks,
            "breakdown": category_breakdown,
            "system_status": "Protected"
        }


# --- VERIFICATION SUITE (UNIT TESTS) ---
if __name__ == "__main__":
    print("// INITIATING SHIELD ENGINE VERIFICATION PROTOCOL...")
    shield = UtahShieldEngine(db_path="test_shield.db")
    
    # Test Case 1: Validate blocking of a known malicious tracking script
    test_url_1 = "https://trackingscript.xyz/analytics/gather.js"
    is_blocked, status = shield.inspect_request("https://example.com", test_url_1)
    assert is_blocked == True, "Verification Error: Failed to drop explicit tracker domain."
    print(f"[SUCCESS] Resource verification pass: Verified dropped address -> {test_url_1}")

    # Test Case 2: Validate allowance of safe functional assets
    test_url_2 = "https://images.example.com/assets/logo.png"
    is_blocked, status = shield.inspect_request("https://example.com", test_url_2)
    assert is_blocked == False, "Verification Error: Accidentally blocked a safe asset."
    print(f"[SUCCESS] Resource verification pass: Verified allowed address -> {test_url_2}")

    # Test Case 3: Validate layout protection loop
    dirty_html = "<html><body><script>window.open('https://popads.net');</script></body></html>"
    clean_html, count = shield.process_page_layout(dirty_html)
    assert count > 0, "Verification Error: Failed to neutralize popup window code."
    print(f"[SUCCESS] Layout protection pass: Neutralized {count} malicious structural segments.")

    # Display final metric summary dashboard data
    metrics = shield.get_dashboard_metrics()
    print("\n" + "="*40)
    print("LIVE SHIELD DASHBOARD METRICS:")
    print(json.dumps(metrics, indent=2))
    print("="*40)

    # Clean up test environment database
    del shield
    import gc
    gc.collect()
    time.sleep(0.1)
    if os.path.exists("test_shield.db"):
        try:
            os.remove("test_shield.db")
        except PermissionError:
            pass
