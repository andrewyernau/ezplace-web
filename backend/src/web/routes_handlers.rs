// web/routes_handlers.rs
use axum::response::Html;
use tower_cookies::{Cookie, Cookies};

use crate::web::minecraft_query::query_minecraft_server;
use crate::models::{Players, ServerStats};
use crate::web::AUTH_TOKEN;
use crate::{Error, Result};
use reqwest::header::USER_AGENT;

// Function to fetch Minecraft server status
// async fn fetch_server_status(host: &str) -> Result<ServerStats> {
//     let url = format!("https://api.mcsrvstat.us/2/{}", host);
    
//     // Create a client with proper timeout settings
//     let client = reqwest::Client::builder()
//         .timeout(std::time::Duration::from_secs(10))
//         .build()
//         .map_err(|e| Error::ServerError(format!("Client build error: {}", e)))?;
    
//     // Make the request with User-Agent header
//     let response = client
//         .get(&url)
//         .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
//         .send()
//         .await
//         .map_err(|e| Error::ServerError(format!("Request error: {}", e)))?;
    
//     // Check status code first
//     if !response.status().is_success() {
//         return Err(Error::ServerError(format!("API returned error status: {}", response.status())));
//     }
    
//     let response_text = response
//         .text()
//         .await
//         .map_err(|e| Error::ServerError(format!("Response text error: {}", e)))?;
    
//     // Debug output - remove in production
//     // println!("API Response: {}", &response_text);
    
//     if response_text.is_empty() {
//         return Err(Error::ServerError("Empty response from server".to_string()));
//     }
    
//     // Try parsing the JSON with error handling
//     let json: serde_json::Value = match serde_json::from_str(&response_text) {
//         Ok(json) => json,
//         Err(e) => {
//             // Log the first portion of the response for debugging
//             let preview = if response_text.len() > 100 {
//                 format!("{}...", &response_text[0..100])
//             } else {
//                 response_text.clone()
//             };
//             eprintln!("JSON parse error: {}. Response preview: {}", e, preview);
//             return Err(Error::ServerError(format!("Failed to parse JSON: {}", e)));
//         }
//     };
    
//     // Field extraction
//     let online = json
//         .get("online")
//         .and_then(|v| v.as_bool())
//         .unwrap_or(false);
    
//     // Version info
//     let protocol_name = json
//         .get("protocol_name")
//         .and_then(|v| v.as_str())
//         .or_else(|| json.get("version").and_then(|v| v.as_str()))
//         .unwrap_or("Unknown");
    
//     // Get player info
//     let players_online = json
//         .get("players")
//         .and_then(|p| p.get("online"))
//         .and_then(|v| v.as_u64())
//         .unwrap_or(0) as u32;
    
//     let players_max = json
//         .get("players")
//         .and_then(|p| p.get("max"))
//         .and_then(|v| v.as_u64())
//         .unwrap_or(0) as u32;
    
//     Ok(ServerStats {
//         online,
//         protocol_name: protocol_name.to_string(),
//         players: Players {
//             online: players_online,
//             max: players_max,
//         },
//     })
// }

// Return the login form HTML
pub async fn get_login_form() -> Html<String> {
    let html = r#"
        <span class='close'>&times;</span>
        <h2>Login with your Minecraft username</h2>
        <form id='login-form' hx-post='/api/session' hx-target='#auth-section' hx-swap='outerHTML'>
            <div class='form-group'>
                <label for='minecraft-username'>Minecraft Username</label>
                <input type='text' id='minecraft-username' name='username' required>
            </div>
            <button type='submit' class='submit-btn'>Login</button>
        </form>
        "#;

    Html(html.to_string())
}

// Get server statistics HTML
pub async fn get_server_stats() -> Result<Html<String>> {
    let host = "192.168.0.16";
    let port = 25560; // Puerto estándar de Minecraft
    
    // Usa la nueva función que implementamos para consultar directamente al servidor
    match query_minecraft_server(host, port) {
        Ok(status) => {
            let status_class = if status.online {
                "status-online"
            } else {
                "status-offline"
            };
            let status_text = if status.online { "Online" } else { "Offline" };
            let html = format!(
                r#"
                    <div class="stats-placeholder">
                        <div class="stat">
                            <h3>Version</h3>
                            <p>{}</p>
                        </div>
                        <div class="stat">
                            <h3>Players Online</h3>
                            <p>{}/{}</p>
                        </div>
                        <div class="stat">
                            <h3>Status</h3>
                            <p class="{}">{}</p>
                        </div>
                    </div>
                    "#,
                status.protocol_name,
                status.players.online,
                status.players.max,
                status_class,
                status_text
            );
            Ok(Html(html))
        }
        Err(err) => {
            eprintln!("Error querying server status: {:?}", err);
            let html = format!(
                r#"
                    <div class="stats-placeholder">
                        <div class="stat">
                            <h3>Version</h3>
                            <p>Unknown</p>
                        </div>
                        <div class="stat">
                            <h3>Players Online</h3>
                            <p>0/0</p>
                        </div>
                        <div class="stat">
                            <h3>Status</h3>
                            <p class="status-offline">Offline</p>
                        </div>
                    </div>
                    "#
            );
            Ok(Html(html))
        }
    }
}

pub async fn get_read_more_content() -> Html<String> {
    let html = r#"
        <span class='close'>&times;</span>
        <h2>EzPlace Network Opens its Doors!</h2>
        <div class="modal-image">
            <img src="assets/images/announcements.webp" alt="Server Opening Celebration">
        </div>
        <div class="modal-description">
    <p>We are thrilled to announce the **beta launch** of the EzPlace Network Survival mode!</p>

    <p>After months of preparation, optimization, and crafting unique features, our Survival server is now open to all players.</p>

    <h3>What you can find on our server right now:</h3>
    <ul>
        <li>Survival gameplay with vanilla-friendly add-ons</li>
        <li>Custom UHC minigame</li>
        <li>Global economy system</li>
    </ul>

    <h3>What makes our Survival mode special:</h3>
    <ul>
        <li>A well-organized world system to encourage community mega-build projects</li>
        <li>Private group chats (Currently available: English and Spanish — you can request a custom one!)</li>
        <li>Custom mob effect potions</li>
        <li>An interactive custom world web map: <a href="https://maps.ezplace.net">maps.ezplace.net</a></li>
    </ul>

    <h3>Future plans:</h3>
    <ul>
        <li>Weekly rotating minigames: One game mode locks, another unlocks</li>
        <li>Use the global economy to purchase a temporary pass to access locked game modes</li>
        <li>An incredible story mode in our upcoming RPG server</li>
        <li>If everything goes well: Modded Survival and Survival Anarchy!</li>
    </ul>

    <p>The server is currently running on Minecraft version 1.21.4, with plans to upgrade to 1.21.6 to support upcoming features.</p>

    <h3>Join us today!</h3>
    <p>Simply connect to <strong>play.ezplace.net</strong> and begin your adventure. You can also join our Discord community to stay updated on server news and events.</p>

    <div class="modal-cta">
        <a href="https://discord.gg/939UecsD95" class="social-link">Join our Discord</a>
    </div>
</div>

        "#;

    Html(html.to_string())
}

// For get_contact_content
pub async fn get_contact_content() -> Html<String> {
    let html = r#"
        <span class='close'>&times;</span>
        <h2>Contact EzPlace Network</h2>
        <div class="modal-description">
            <p>We value your feedback and are always here to help! Feel free to reach out to us with any questions, suggestions, or concerns.</p>
            
            <h3>Contact Methods:</h3>
            <ul>
                <li>Email: <a href="mailto:contact@ezplace.net">contact@ezplace.net</a></li>
                <li>Discord: <a href="https://discord.gg/939UecsD95">Join our Discord server</a></li>
            </ul>
            
            <h3>Support Hours:</h3>
            <p>Our team is available to assist you:</p>
            <ul>
                <li>Monday - Friday: 10:00 AM - 8:00 PM (CET)</li>
                <li>Saturday: 12:00 PM - 6:00 PM (CET)</li>
                <li>Sunday: Limited support via Discord only</li>
            </ul>
            
            <h3>Report Issues:</h3>
            <p>If you encounter any bugs or technical issues while playing on our server, please provide the following information:</p>
            <ul>
                <li>Your Minecraft username</li>
                <li>Server you were playing on (game mode)</li>
                <li>Description of the issue</li>
                <li>Steps to reproduce (if applicable)</li>
                <li>Screenshots (if available)</li>
            </ul>
            
            <div class="modal-cta">
                <a href="https://discord.gg/939UecsD95" class="social-link">Join our Discord</a>
            </div>
        </div>
        "#;

    Html(html.to_string())
}

// For get_community_content
pub async fn get_community_content() -> Html<String> {
    let html = r#"
        <span class='close'>&times;</span>
        <h2>Join Our Growing Community</h2>
        <div class="modal-description">
            <p>The EzPlace Network is more than just a Minecraft server – it's a thriving community of players from around the world!</p>
            
            <h3>Community Events:</h3>
            <ul>
                <li>Weekly building contests with in-game rewards</li>
                <li>Monthly PvP tournaments</li>
                <li>Special holiday celebrations</li>
                <li>Community build projects</li>
            </ul>
            
            <h3>Community Guidelines:</h3>
            <p>To ensure a positive experience for everyone, we ask all players to follow these simple guidelines:</p>
            <ul>
                <li>Be respectful to all players and staff</li>
                <li>No griefing, stealing, or destroying others' builds</li>
                <li>No hacking, cheating, or using unfair advantages</li>
                <li>Keep chat appropriate and friendly</li>
                <li>Have fun and help others enjoy their time too!</li>
            </ul>
            
            <h3>Community Spotlight:</h3>
            <p>Each month, we feature outstanding community members who contribute positively to our server. Winners receive special perks and recognition on our Discord and website!</p>
            
            <h3>Join Our Social Platforms:</h3>
            <ul>
                <li>Discord: <a href="https://discord.gg/939UecsD95">Join our server</a></li>
                <li>Instagram: <a href="https://instagram.com/ezplacenetwork">@ezplacenetwork</a></li>
                <li>Twitter: <a href="https://twitter.com/ezplacenet">@ezplacenet</a></li>
            </ul>
            
            <div class="modal-cta">
                <a href="https://discord.gg/939UecsD95" class="social-link">Join our Discord</a>
            </div>
        </div>
        "#;

    Html(html.to_string())
}

// For get_cookies_content with updated format and cookie banner mention
pub async fn get_cookies_content() -> Html<String> {
    let html = r#"
        <span class='close'>&times;</span>
        <h2>Cookies Policy</h2>
        <div class="modal-description">
            <p>Last updated: April 19, 2025</p>
            
            <p>This Cookies Policy explains what Cookies are and how We use them. You should read this policy so You can understand what type of cookies We use, or the information We collect using Cookies and how that information is used.</p>
            
            <p>Our website uses a cookie banner/popup that appears when you first visit our site to inform you about our use of cookies and to obtain your consent where required by applicable law.</p>
            
            <h3>Interpretation and Definitions</h3>
            
            <h4>Interpretation</h4>
            <p>The words of which the initial letter is capitalized have meanings defined under the following conditions. The following definitions shall have the same meaning regardless of whether they appear in singular or in plural.</p>
            
            <h4>Definitions</h4>
            <p>For the purposes of this Cookies Policy:</p>
            <ul>
                <li><strong>Company</strong> (referred to as either "the Company", "We", "Us" or "Our" in this Cookies Policy) refers to EzPlace Network.</li>
                <li><strong>Cookies</strong> means small files that are placed on Your computer, mobile device or any other device by a website, containing details of your browsing history on that website among its many uses.</li>
                <li><strong>Website</strong> refers to EzPlace Network, accessible from <a href="https://www.ezplace.net" rel="external nofollow noopener" target="_blank">https://www.ezplace.net</a></li>
                <li><strong>You</strong> means the individual accessing or using the Website, or a company, or any legal entity on behalf of which such individual is accessing or using the Website, as applicable.</li>
            </ul>
            
            <h3>The use of the Cookies</h3>
            
            <h4>Type of Cookies We Use</h4>
            <p>Cookies can be "Persistent" or "Session" Cookies. Persistent Cookies remain on your personal computer or mobile device when You go offline, while Session Cookies are deleted as soon as You close your web browser.</p>
            <p>We use both session and persistent Cookies for the purposes set out below:</p>
            <ul>
                <li>
                    <p><strong>Necessary / Essential Cookies</strong></p>
                    <p>Type: Session Cookies</p>
                    <p>Administered by: Us</p>
                    <p>Purpose: These Cookies are essential to provide You with services available through the Website and to enable You to use some of its features. They help to authenticate users and prevent fraudulent use of user accounts. Without these Cookies, the services that You have asked for cannot be provided, and We only use these Cookies to provide You with those services.</p>
                </li>
                <li>
                    <p><strong>Functionality Cookies</strong></p>
                    <p>Type: Persistent Cookies</p>
                    <p>Administered by: Us</p>
                    <p>Purpose: These Cookies allow us to remember choices You make when You use the Website, such as remembering your login details or language preference. The purpose of these Cookies is to provide You with a more personal experience and to avoid You having to re-enter your preferences every time You use the Website.</p>
                </li>
            </ul>
            
            <h3>Your Choices Regarding Cookies</h3>
            <p>If You prefer to avoid the use of Cookies on the Website, first You must disable the use of Cookies in your browser and then delete the Cookies saved in your browser associated with this website. You may use this option for preventing the use of Cookies at any time.</p>
            <p>If You do not accept Our Cookies, You may experience some inconvenience in your use of the Website and some features may not function properly.</p>
            <p>If You'd like to delete Cookies or instruct your web browser to delete or refuse Cookies, please visit the help pages of your web browser.</p>
            <ul>
                <li>For the Chrome web browser, please visit this page from Google: <a href="https://support.google.com/accounts/answer/32050" rel="external nofollow noopener" target="_blank">https://support.google.com/accounts/answer/32050</a></li>
                <li>For the Internet Explorer web browser, please visit this page from Microsoft: <a href="http://support.microsoft.com/kb/278835" rel="external nofollow noopener" target="_blank">http://support.microsoft.com/kb/278835</a></li>
                <li>For the Firefox web browser, please visit this page from Mozilla: <a href="https://support.mozilla.org/en-US/kb/delete-cookies-remove-info-websites-stored" rel="external nofollow noopener" target="_blank">https://support.mozilla.org/en-US/kb/delete-cookies-remove-info-websites-stored</a></li>
                <li>For the Safari web browser, please visit this page from Apple: <a href="https://support.apple.com/guide/safari/manage-cookies-and-website-data-sfri11471/mac" rel="external nofollow noopener" target="_blank">https://support.apple.com/guide/safari/manage-cookies-and-website-data-sfri11471/mac</a></li>
            </ul>
            <p>For any other web browser, please visit your web browser's official web pages.</p>
            
            <h3>Contact Us</h3>
            <p>If you have any questions about this Cookies Policy, You can contact us:</p>
            <ul>
                <li>By email: <a href="mailto:contact@ezplace.net">contact@ezplace.net</a></li>
            </ul>
            
            <div class="modal-cta">
                <a href="mailto:contact@ezplace.net" class="social-link">Contact Us</a>
            </div>
        </div>
        "#;

    Html(html.to_string())
}

// For get_terms_content with updated format to match read_more_content
pub async fn get_terms_content() -> Html<String> {
    let html = r#"
        <span class='close'>&times;</span>
        <h2>Privacy Policy</h2>
        <div class="modal-description">
            <p>Last updated: April 19, 2025</p>
            
            <p>This Privacy Policy describes Our policies and procedures on the collection, use and disclosure of Your information when You use the Service and tells You about Your privacy rights and how the law protects You.</p>
            
            <p>We use Your Personal data to provide and improve the Service. By using the Service, You agree to the collection and use of information in accordance with this Privacy Policy.</p>
            
            <h3>Interpretation and Definitions</h3>
            
            <h4>Interpretation</h4>
            <p>The words of which the initial letter is capitalized have meanings defined under the following conditions. The following definitions shall have the same meaning regardless of whether they appear in singular or in plural.</p>
            
            <h4>Definitions</h4>
            <p>For the purposes of this Privacy Policy:</p>
            <ul>
                <li><strong>Account</strong> means a unique account created for You to access our Service or parts of our Service.</li>
                <li><strong>Affiliate</strong> means an entity that controls, is controlled by or is under common control with a party, where "control" means ownership of 50% or more of the shares, equity interest or other securities entitled to vote for election of directors or other managing authority.</li>
                <li><strong>Company</strong> (referred to as either "the Company", "We", "Us" or "Our" in this Agreement) refers to EzPlace Network.</li>
                <li><strong>Cookies</strong> are small files that are placed on Your computer, mobile device or any other device by a website, containing the details of Your browsing history on that website among its many uses.</li>
                <li><strong>Country</strong> refers to: Spain</li>
                <li><strong>Device</strong> means any device that can access the Service such as a computer, a cellphone or a digital tablet.</li>
                <li><strong>Personal Data</strong> is any information that relates to an identified or identifiable individual.</li>
                <li><strong>Service</strong> refers to the Website.</li>
                <li><strong>Service Provider</strong> means any natural or legal person who processes the data on behalf of the Company. It refers to third-party companies or individuals employed by the Company to facilitate the Service, to provide the Service on behalf of the Company, to perform services related to the Service or to assist the Company in analyzing how the Service is used.</li>
                <li><strong>Usage Data</strong> refers to data collected automatically, either generated by the use of the Service or from the Service infrastructure itself (for example, the duration of a page visit).</li>
                <li><strong>Website</strong> refers to EzPlace Network, accessible from <a href="https://www.ezplace.net" rel="external nofollow noopener" target="_blank">https://www.ezplace.net</a></li>
                <li><strong>You</strong> means the individual accessing or using the Service, or the company, or other legal entity on behalf of which such individual is accessing or using the Service, as applicable.</li>
            </ul>
            
            <h3>Collecting and Using Your Personal Data</h3>
            
            <h4>Types of Data Collected</h4>
            
            <h5>Personal Data</h5>
            <p>While using Our Service, We may ask You to provide Us with certain personally identifiable information that can be used to contact or identify You. Personally identifiable information may include, but is not limited to:</p>
            <ul>
                <li>Email address</li>
                <li>First name and last name</li>
                <li>Usage Data</li>
            </ul>
            
            <h5>Usage Data</h5>
            <p>Usage Data is collected automatically when using the Service.</p>
            <p>Usage Data may include information such as Your Device's Internet Protocol address (e.g. IP address), browser type, browser version, the pages of our Service that You visit, the time and date of Your visit, the time spent on those pages, unique device identifiers and other diagnostic data.</p>
            
            <h3>Security of Your Personal Data</h3>
            <p>The security of Your Personal Data is important to Us, but remember that no method of transmission over the Internet, or method of electronic storage is 100% secure. While We strive to use commercially acceptable means to protect Your Personal Data, We cannot guarantee its absolute security.</p>
            
            <h3>Children's Privacy</h3>
            <p>Our Service does not address anyone under the age of 13. We do not knowingly collect personally identifiable information from anyone under the age of 13. If You are a parent or guardian and You are aware that Your child has provided Us with Personal Data, please contact Us.</p>
            
            <h3>Links to Other Websites</h3>
            <p>Our Service may contain links to other websites that are not operated by Us. If You click on a third party link, You will be directed to that third party's site. We strongly advise You to review the Privacy Policy of every site You visit.</p>
            
            <h3>Changes to this Privacy Policy</h3>
            <p>We may update Our Privacy Policy from time to time. We will notify You of any changes by posting the new Privacy Policy on this page.</p>
            
            <h3>Contact Us</h3>
            <p>If you have any questions about this Privacy Policy, You can contact us:</p>
            <ul>
                <li>By email: <a href="mailto:contact@ezplace.net">contact@ezplace.net</a></li>
            </ul>
            
            <div class="modal-cta">
                <a href="mailto:contact@ezplace.net" class="social-link">Contact Us</a>
            </div>
        </div>
        "#;

    Html(html.to_string())
}

// Update the logout function to clear the auth cookie
pub async fn logout(cookies: Cookies) -> Html<String> {
    // Create a cookie with the same name but empty value, path and with negative max_age
    let mut cookie = Cookie::new(AUTH_TOKEN, "");
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_max_age(time::Duration::seconds(-1));

    // Add/replace the cookie (effectively removing it)
    cookies.add(cookie);

    println!("->> {:<12} - logout - Cookie removed", "HANDLER");

    Html(r#"
        <div id='auth-section'>
            <button id='login-btn' class='login-btn' hx-get='/api/login-form' hx-target='#login-modal-content' hx-trigger='click' onclick="document.getElementById('login-modal').style.display='block'">Login</button>
        </div>
        "#.to_string())
}
