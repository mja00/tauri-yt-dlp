# macOS Code Signing and Notarization Setup Guide

This guide walks you through setting up proper code signing and notarization for your Tauri macOS app, following the [tauri-action example](https://github.com/tauri-apps/tauri-action/blob/dev/examples/publish-to-auto-release-universal-macos-app-with-signing-certificate.yml).

## Prerequisites

1. **Apple Developer Account** - You need an active Apple Developer Program membership ($99/year)
   - Sign up at: https://developer.apple.com/programs/
   - This is required to obtain Developer ID certificates for code signing

## Step 1: Create a Developer ID Application Certificate

1. **Log in to Apple Developer Portal**
   - Go to: https://developer.apple.com/account
   - Navigate to **Certificates, Identifiers & Profiles**

2. **Create a Certificate Signing Request (CSR)**
   - On your Mac, open **Keychain Access** (Applications > Utilities)
   - Go to **Keychain Access** > **Certificate Assistant** > **Request a Certificate From a Certificate Authority**
   - Enter your email address and name
   - Select **"Save to disk"**
   - Click **Continue** and save the `.certSigningRequest` file

3. **Create the Developer ID Application Certificate**
   - In Apple Developer Portal, click the **+** button to create a new certificate
   - Select **Developer ID Application** (under the "Software" section)
   - Click **Continue**
   - Upload the CSR file you just created
   - Click **Continue** and then **Download** the certificate
   - Double-click the downloaded `.cer` file to install it in your Keychain

## Step 2: Export the Certificate as a .p12 File

1. **Open Keychain Access**
   - Find your **Developer ID Application** certificate (it should show your name/team)
   - Expand the certificate to see the associated private key
   - Select both the certificate and its private key (hold Cmd to select both)

2. **Export the Certificate**
   - Right-click and select **Export 2 items...**
   - Choose **Personal Information Exchange (.p12)** format
   - Save it with a secure password (you'll need this password later)
   - **Important**: Remember this password - you'll need it for the `APPLE_CERTIFICATE_PASSWORD` secret

3. **Convert to Base64**
   - Open Terminal and run:
     ```bash
     base64 -i /path/to/your/certificate.p12 | pbcopy
     ```
   - This copies the base64-encoded certificate to your clipboard
   - **Save this base64 string** - you'll need it for the `APPLE_CERTIFICATE` secret

## Step 3: Get Your Apple Team ID

1. **Find Your Team ID**
   - Go to: https://developer.apple.com/account
   - Click on **Membership** in the sidebar
   - Your **Team ID** is displayed (format: `ABCDE12345`)
   - **Save this** - you'll need it for the `APPLE_TEAM_ID` secret

## Step 4: Create an App-Specific Password for Notarization

1. **Enable Two-Factor Authentication** (if not already enabled)
   - Go to: https://appleid.apple.com
   - Sign in and ensure 2FA is enabled

2. **Generate App-Specific Password**
   - Go to: https://appleid.apple.com/account/manage
   - Scroll to **App-Specific Passwords**
   - Click **Generate Password...**
   - Give it a name like "GitHub Actions Notarization"
   - Click **Create**
   - **Copy the password immediately** - you'll only see it once
   - **Save this** - you'll need it for the `APPLE_PASSWORD` secret

## Step 5: Configure GitHub Secrets

1. **Navigate to Your Repository Settings**
   - Go to your GitHub repository
   - Click **Settings** > **Secrets and variables** > **Actions**

2. **Add the Following Secrets:**

   | Secret Name | Description | Value |
   |------------|-------------|-------|
   | `APPLE_CERTIFICATE` | Base64-encoded .p12 certificate | The base64 string from Step 2 |
   | `APPLE_CERTIFICATE_PASSWORD` | Password for the .p12 file | The password you set when exporting |
   | `KEYCHAIN_PASSWORD` | Password for the temporary keychain | Any secure password (e.g., generate a random one) |
   | `APPLE_ID` | Your Apple ID email | Your Apple ID email address |
   | `APPLE_PASSWORD` | App-specific password | The app-specific password from Step 4 |
   | `APPLE_TEAM_ID` | Your Apple Team ID | The Team ID from Step 3 |

3. **Adding Secrets:**
   - Click **New repository secret**
   - Enter the secret name and value
   - Click **Add secret**
   - Repeat for all secrets listed above

## Step 6: Verify the Setup

1. **Push to Main Branch or Trigger Workflow**
   - The workflow will automatically:
     - Import your certificate
     - Sign the sidecar binaries (yt-dlp_macos)
     - Sign the main app bundle
     - Notarize the app with Apple
     - Create a GitHub release with the signed app

2. **Check the Workflow Logs**
   - Go to **Actions** tab in your repository
   - Click on the latest workflow run
   - Check for any errors in the signing/notarization steps

3. **Test the Built App**
   - Download the app from the GitHub release
   - Try opening it on an ARM Mac
   - It should open without the "damaged" error

## Troubleshooting

### Certificate Not Found
- **Error**: "No Developer ID Application certificate found"
- **Solution**: Ensure you created a **Developer ID Application** certificate (not "Mac Development" or "Mac App Distribution")

### Notarization Fails
- **Error**: Notarization submission fails
- **Solution**: 
  - Verify your `APPLE_ID` and `APPLE_PASSWORD` are correct
  - Ensure 2FA is enabled on your Apple ID
  - Check that the app-specific password is valid

### Keychain Issues
- **Error**: Keychain access denied
- **Solution**: The workflow creates a temporary keychain, but if issues persist, try changing the `KEYCHAIN_PASSWORD` secret

### App Still Shows as Damaged
- **Solution**: 
  - Ensure all secrets are correctly set
  - Check that the certificate is valid (not expired)
  - Verify the app was notarized successfully in the workflow logs
  - Try removing the quarantine attribute: `xattr -d com.apple.quarantine /path/to/app.app`

## Fallback: Ad-Hoc Signing

If you don't have an Apple Developer account yet, the workflow will automatically fall back to **ad-hoc signing** (`-` identity). This allows the app to run on ARM Macs but:
- Users will see "untrusted developer" warnings
- The app cannot be notarized
- Distribution is less professional

To use ad-hoc signing, simply **don't set** the `APPLE_CERTIFICATE` secret. The workflow will detect this and use ad-hoc signing automatically.

## Additional Resources

- [Tauri macOS Code Signing Guide](https://tauri.app/v1/guides/distribution/sign-macos/)
- [Apple Code Signing Guide](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/)
- [Apple Notarization Documentation](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)

