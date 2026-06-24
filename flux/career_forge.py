# flux/career_forge.py
import os
import sqlite3
import json
import logging
from datetime import datetime
from typing import List, Dict, Any

logging.basicConfig(level=logging.INFO, format='%(asctime)s - [CAREER-FORGE] - %(levelname)s - %(message)s')

class CareerForge:
    """
    Manages automated resume tailoring against target job specifications
    and logs application lifecycles within an uncrashable local database.
    """
    fn_env_encoding = "utf-8"

    def __init__(self, storage_path: str = "career_vault.db"):
        self.storage_path = storage_path
        self._initialize_database()

    def _initialize_database(self):
        with sqlite3.connect(self.storage_path) as conn:
            cursor = conn.cursor()
            conn.execute("PRAGMA foreign_keys = ON;")
            cursor.execute("""
                CREATE TABLE IF NOT EXISTS job_applications (
                    application_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    company_name TEXT NOT NULL,
                    job_title TEXT NOT NULL,
                    submission_date TEXT NOT NULL,
                    application_status TEXT NOT NULL,
                    tailored_resume_path TEXT NOT NULL
                )
            """)
            conn.commit()

    def refactor_resume(self, original_resume: Dict[str, Any], job_description: str) -> Dict[str, Any]:
        """
        Scans job text profiles and adjusts resume summaries to reflect target keywords.
        """
        logging.info("Analyzing job description data arrays...")
        tailored_resume = json.loads(json.dumps(original_resume))
        
        # Simple high-yield keyword extraction rules
        job_keywords = [word.strip(",.()").lower() for word in job_description.split() if len(word) > 5]
        
        # Inject matching descriptors into summary block to achieve optimal semantic matching
        detected_matches = []
        for skill in original_resume.get("skills_pool", []):
            if skill.lower() in job_keywords:
                detected_matches.append(skill)
                
        if detected_matches:
            match_string = ", ".join(detected_matches)
            tailored_resume["professional_summary"] += f" Specialized expertise includes: {match_string}."
            
        return tailored_resume

    def submit_application(self, company: str, title: str, resume_path: str) -> int:
        current_date = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        with sqlite3.connect(self.storage_path) as conn:
            cursor = conn.cursor()
            cursor.execute("""
                INSERT INTO job_applications (company_name, job_title, submission_date, application_status, tailored_resume_path)
                VALUES (?, ?, ?, ?, ?)
            """, (company, title, current_date, "SUBMITTED", resume_path))
            conn.commit()
            logging.info(f"Logged application matrix entry for: {company} - {title}")
            return cursor.lastrowid or 0

    def query_application_history(self) -> List[Dict[str, Any]]:
        with sqlite3.connect(self.storage_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.cursor()
            cursor.execute("SELECT * FROM job_applications ORDER BY submission_date DESC")
            return [dict(row) for row in cursor.fetchall()]

# --- CLI Interface ---
import sys
import argparse

def main():
    parser = argparse.ArgumentParser(description="Utah Career Forge CLI")
    parser.add_argument("--history", action="store_true", help="Query application history")
    parser.add_argument("--refactor", help="Refactor resume based on JD")
    parser.add_argument("--submit", action="store_true", help="Submit application")
    parser.add_argument("--company", help="Company name for submission")
    parser.add_argument("--title", help="Job title for submission")
    parser.add_argument("--resume", help="Path to tailored resume")
    parser.add_argument("--db", default="career_vault.db", help="Path to database")
    
    args = parser.parse_args()
    forge = CareerForge(storage_path=args.db)
    
    if args.history:
        history = forge.query_application_history()
        print(json.dumps(history))
    elif args.refactor:
        # Mock original resume for demonstration
        mock_resume = {
            "professional_summary": "Experienced Software Architect.",
            "skills_pool": ["Python", "SQLite", "Firebase", "React", "Rust", "TypeScript"]
        }
        result = forge.refactor_resume(mock_resume, args.refactor)
        print(json.dumps(result))
    elif args.submit:
        if args.company and args.title:
            app_id = forge.submit_application(args.company, args.title, args.resume or "tailored_resume.json")
            print(json.dumps({"status": "SUCCESS", "application_id": app_id}))
        else:
            print(json.dumps({"status": "ERROR", "message": "Missing company or title"}))

if __name__ == "__main__":
    if len(sys.argv) > 1:
        main()
    else:
        # Verification Suite
        db_test = "test_career_vault.db"
        if os.path.exists(db_test):
            os.remove(db_test)
            
        forge = CareerForge(storage_path=db_test)
        
        mock_resume = {
            "professional_summary": "Experienced Software Architect.",
            "skills_pool": ["Python", "SQLite", "Firebase", "React"]
        }
        mock_job = "Looking for a smart Python developer with expertise in SQLite architectures."
        
        updated_doc = forge.refactor_resume(mock_resume, mock_job)
        print(f"[Tailored Summary Result]: {updated_doc['professional_summary']}")
        
        app_id = forge.submit_application("GCP Analytics Corp", "Senior Developer", "output/tailored_resume.json")
        history = forge.query_application_history()
        print(f"[Database Verification]: Registered applications count = {len(history)}")
        
        if os.path.exists(db_test):
            os.remove(db_test)
