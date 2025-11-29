
import streamlit as st
import streamlit_authenticator as stauth
import sqlite3
import bcrypt
import yaml
from yaml.loader import SafeLoader
from datetime import datetime
import pandas as pd
import frontend.app as main_app

DB_FILE = 'users.db'

# ✅ Set page config ONCE at the top
st.set_page_config(
        page_title="Login",
        page_icon="🔑",
        layout="wide",
        initial_sidebar_state="expanded",
    )

# -------------------- Load Config --------------------
with open('user.yaml') as file:
    config = yaml.load(file, Loader=SafeLoader)

# -------------------- Database Functions --------------------
def init_db():
    with sqlite3.connect(DB_FILE, timeout=10) as conn:
        c = conn.cursor()
        c.execute('''CREATE TABLE IF NOT EXISTS users (
                        username TEXT PRIMARY KEY,
                        name TEXT,
                        hashed_password TEXT,
                        role TEXT,
                        email TEXT)''')
        c.execute('''CREATE TABLE IF NOT EXISTS audit_logs (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        timestamp TEXT,
                        username TEXT,
                        action TEXT)''')

        # Insert users from config.yaml if not in DB
        for username, details in config['credentials']['usernames'].items():
            c.execute("SELECT * FROM users WHERE username=?", (username,))
            if not c.fetchone():
                c.execute("INSERT INTO users VALUES (?, ?, ?, ?, ?)",
                          (username, details['name'], details['password'], details['role'], details['email']))
        conn.commit()

        conn.commit()

def load_credentials():
    conn = sqlite3.connect("users.db")
    cur = conn.cursor()
    cur.execute("SELECT username, name, hashed_password FROM users")
    rows = cur.fetchall()
    conn.close()

    credentials = {"usernames": {}}
    for username, name, pwd_hash in rows:
        credentials["usernames"][username] = {
            "name": name,
            "failed_login_attempts": 0,
            "logged_in": False, 
            "password": pwd_hash,
        }
    return credentials

def log_action(username, action):
    with sqlite3.connect(DB_FILE, timeout=10) as conn:
        c = conn.cursor()
        c.execute("INSERT INTO audit_logs (timestamp, username, action) VALUES (?, ?, ?)",
                  (datetime.now().strftime("%Y-%m-%d %H:%M:%S"), username, action))
        conn.commit()

def get_user_role(username):
    with sqlite3.connect(DB_FILE, timeout=10) as conn:
        c = conn.cursor()
        c.execute("SELECT role FROM users WHERE username=?", (username,))
        result = c.fetchone()
        return result[0] if result else None

def get_user_info(username):
    with sqlite3.connect(DB_FILE, timeout=10) as conn:
        c = conn.cursor()
        c.execute("SELECT name, email, role FROM users WHERE username=?", (username,))
        return c.fetchone()

def get_users():
    with sqlite3.connect(DB_FILE, timeout=10) as conn:
        c = conn.cursor()
        c.execute("SELECT username, name, role, email FROM users")
        return c.fetchall()

def add_user(username, name, password, role, email):
    with sqlite3.connect(DB_FILE, timeout=10) as conn:
        c = conn.cursor()
        hashed_pw = bcrypt.hashpw(password.encode(), bcrypt.gensalt()).decode()
        try:
            c.execute("INSERT INTO users VALUES (?, ?, ?, ?, ?)", (username, name, hashed_pw, role, email))
            conn.commit()
            log_action(username, f"User {username} created")
            return True
        except sqlite3.IntegrityError:
            return False

def reset_password(username, new_password):
    with sqlite3.connect(DB_FILE, timeout=10) as conn:
        c = conn.cursor()
        hashed_pw = bcrypt.hashpw(new_password.encode(), bcrypt.gensalt()).decode()
        c.execute("UPDATE users SET hashed_password=? WHERE username=?", (hashed_pw, username))
        conn.commit()
        log_action(username, "Password reset")

def get_audit_logs():
    with sqlite3.connect(DB_FILE, timeout=10) as conn:
        c = conn.cursor()
        c.execute("SELECT timestamp, username, action FROM audit_logs ORDER BY id DESC LIMIT 50")
        return c.fetchall()

# -------------------- Initialize DB --------------------
init_db()

# -------------------- Load users from DB --------------------
users = get_users()
names = [u[1] for u in users]
usernames = [u[0] for u in users]
hashed_passwords = [u[2] for u in users]

# -------------------- Authenticator --------------------
def get_authenticator():
    credentials = load_credentials()
    return stauth.Authenticate(
        credentials,
        config['cookie']['name'],
        config['cookie']['key'],
        config['cookie']['expiry_days']

    )
# -------------------- Session State --------------------
# Force login page on restart
if "page" not in st.session_state:
    st.session_state["page"] = "Login"
if "username" not in st.session_state:
    st.session_state["username"] = None
if "login_logged" not in st.session_state:
    st.session_state["login_logged"] = False
if "prev_auth_status" not in st.session_state:
    st.session_state["prev_auth_status"] = None
if "authenticator" not in st.session_state:
    st.session_state["authenticator"] = get_authenticator()
authenticator = st.session_state["authenticator"]
st.sidebar.header("🏺 Archaeological Artifact Identifier")


# -------------------- Login Page --------------------
try:
    authenticator.login(location='main')
except stauth.LoginError as e:
    st.error(e)



if st.session_state['authentication_status']:
    # Dynamic title based on page
    page_title = f"{st.session_state['page']} | Admin Panel" if get_user_role(st.session_state["username"]) == 'admin' else f"{st.session_state['page']} | User Dashboard"
    #st.set_page_config(page_title=page_title, page_icon="✅", layout="wide", initial_sidebar_state="expanded")
    #st.title(page_title)
    st.sidebar.success(f'Welcome {st.session_state["name"]}!')
    authentication_status = st.session_state['authentication_status'] 
    # ✅ Log login only once
    if authentication_status and st.session_state["prev_auth_status"] != True:
        log_action(st.session_state['username'], "Logged in")
        st.session_state["login_logged"] = True

    st.session_state["prev_auth_status"] =  st.session_state['authentication_status']
    role = get_user_role(st.session_state["username"])

    # Role-based navigation
    if role == 'admin':
        st.sidebar.title("Admin Menu")
        if st.sidebar.button("Dashboard"):
            st.session_state["page"] = "Admin Dashboard"
        if st.sidebar.button("Gallery Artifacts"):
            st.session_state["page"] = "Gallery Artifacts"
        if st.sidebar.button("User Management"):
            st.session_state["page"] = "User Management"
        if st.sidebar.button("Audit Logs"):
            st.session_state["page"] = "Audit Logs"
    elif  role == 'user':
        st.sidebar.title("User Menu")
        if st.sidebar.button("Dashboard"):
            st.session_state["page"] = "User Dashboard"
        if st.sidebar.button("Upload Artifacts"):
            st.session_state["page"] = "Upload Artifacts"
        if st.sidebar.button("Change Password"):
            st.session_state["page"] = "Change Password"
    elif  role == 'field':
        st.sidebar.title("Field Engineer Menu")
        if st.sidebar.button("Dashboard"):
            st.session_state["page"] = "User Dashboard"
        if st.sidebar.button("Upload Artifacts"):
            st.session_state["page"] = "Upload Artifacts"
        if st.sidebar.button("Gallery Artifacts"):
            st.session_state["page"] = "Gallery Artifacts"
        if st.sidebar.button("Change Password"):
            st.session_state["page"] = "Change Password"
    elif  role == 'onsite':
        st.sidebar.title("Onsite Engineer User Menu")
        if st.sidebar.button("Dashboard"):
            st.session_state["page"] = "User Dashboard"
        if st.sidebar.button("Gallery Artifacts"):
            st.session_state["page"] = "Gallery Artifacts"
        if st.sidebar.button("Change Password"):
            st.session_state["page"] = "Change Password"

# -------------------- Admin Dashboard --------------------
    if st.session_state["page"] == "Admin Dashboard":
        st.header("📊 Admin Dashboard")
        users = get_users()
        st.metric("Total Users", len(users))
        st.metric("Recent Actions", len(get_audit_logs()))

        # Role distribution chart
        role_counts = {}
        for u in users:
            role_counts[u[2]] = role_counts.get(u[2], 0) + 1
        st.subheader("User Role Distribution")
        st.bar_chart(pd.DataFrame.from_dict(role_counts, orient='index', columns=['Count']))

    # -------------------- Admin: Gallery Artifacts --------------------
    elif st.session_state["page"] == "Gallery Artifacts":
        # Render main app
        main_app.main(role)

    # -------------------- Admin: User Management --------------------
    elif st.session_state["page"] == "User Management":
        st.header("👥 Admin - User Management")
        users = get_users()
        search_query = st.text_input("Search by username or email")
        filtered_users = [u for u in users if search_query.lower() in u[0].lower() or search_query.lower() in u[3].lower()]
        st.write("### Current Users")
        for u in filtered_users:
            st.write(f"- {u[0]} | {u[1]} | {u[2]} | {u[3]}")

        st.subheader("Add New User")
        with st.form("add_user_form"):
            new_username = st.text_input("Username")
            new_name = st.text_input("Full Name")
            new_password = st.text_input("Password", type="password")
            new_role = st.selectbox("Role", ["user", "admin", "field","onsite"])
            new_email = st.text_input("Email")
            submitted = st.form_submit_button("Add User")
            if submitted:
                if add_user(new_username, new_name, new_password, new_role, new_email):
                    st.success(f"User {new_username} added successfully!")
                    st.session_state["authenticator"] = get_authenticator()
                else:
                    st.error("Username already exists!")

    # -------------------- Admin: Audit Logs --------------------
    elif st.session_state["page"] == "Audit Logs":
        st.header("📜 Audit Logs (Last 50 Actions)")
        logs = get_audit_logs()
        df_logs = pd.DataFrame(logs, columns=["Timestamp", "Username", "Action"])
        st.dataframe(df_logs)

    # -------------------- User Dashboard --------------------
    elif st.session_state["page"] == "User Dashboard":
        st.header("✅ User Dashboard")
        user_info = get_user_info(st.session_state["username"])
        columns = [ 'name', 'email', 'role' ]
        user_info = dict(zip(columns, user_info))
        st.write(f"**Name:** {user_info['name']}")
        st.write(f"**Email:** {user_info['email']}")
        st.write(f"**Role:** {user_info['role']}")

        st.subheader("Your Recent Activities")
        logs = [log for log in get_audit_logs() if log[1] == st.session_state["username"]]
        df_user_logs = pd.DataFrame(logs, columns=["Timestamp", "Username", "Action"])
        st.dataframe(df_user_logs)

        if logs:
            dates = [log[0].split(" ")[0] for log in logs]
            date_counts = pd.Series(dates).value_counts()
            st.subheader("Activity Trend")
            st.bar_chart(date_counts)

    # -------------------- Admin: Gallery Artifacts --------------------
    elif st.session_state["page"] == "Upload Artifacts":
        # Render main app
        main_app.main(role)
    # -------------------- User: Change Password --------------------
    elif st.session_state["page"] == "Change Password":
        st.header("🔑 Change Password")
        with st.form("change_pw_form"):
            new_password = st.text_input("New Password", type="password")
            submitted = st.form_submit_button("Update Password")
            if submitted:
                reset_password(st.session_state["username"], new_password)
                st.success("Password updated successfully!")

    # -------------------- User: Logout --------------------
    authenticator.logout(location='sidebar')

elif st.session_state['authentication_status'] is False:
    st.error("Username or password is incorrect.")
#    st.session_state.clear()  # Clears all session data
    st.session_state["page"] = "Login"
elif st.session_state['authentication_status'] is None:
    st.warning("Please enter your username and password.")
 #   st.session_state.clear()  # Clears all session data
    st.session_state["page"] = "Login"
#if st.session_state["page"] == "Login":
#    st.session_state["prev_auth_status"] =  None
