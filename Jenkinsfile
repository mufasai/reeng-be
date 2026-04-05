pipeline {

    agent any

    triggers {
        githubPush() 
    }

    environment {
        ENV = '.env'
        IMAGE = 'g3n1k/reengineering-tool-be'
        IMAGEPRE = 'reengineering-tool-be-pre'
        TAG = 'latest'
        REPO = 'https://github.com/Smartelco/reengineering-tool-be.git'

        BRANCH = 'main'
        CRED_ID = 'Smartelco-IT'
        DOCKER_ID = 'test-cicd-docker'
        LOG_FILE = "jenkins_error.log"
                
    }

    stages {

         stage('start build'){
            steps {
                script {
                    sendTelegramMessage("⚙️ Start Build ${IMAGE}:${TAG}")
                }
            }
        }

        stage('clone repo') {
            steps {
                script {
                    try {
                        git branch: "${BRANCH}", credentialsId: "${CRED_ID}", url: "${REPO}"
                    } catch (Exception e) {
                        // writeFile file: LOG_FILE, text: "Error cloning repo: ${e.toString()}\n"
                        def log = currentBuild.rawBuild.getLog(100).join("\n")
                        def message = URLEncoder.encode("🚨 Error cloning repo: ${e.toString()}\n\n${log}", "UTF-8")
                        sendTelegramMessageError(message)
                        
                        error("❌ Gagal clone repo. Log telah disimpan.")
                    }
                }
            }
        }

        // apakah ada perubahan pada Cargo.toml
        // check dengan command git apakah Cargo toml diubah
        // jika diubah, ubah env.CARGO_CHANGED = true
        stage('check Cargo.toml changes') {
            steps {
                script {
                    def changedFiles = sh(
                        script: "git diff --name-only HEAD~1 HEAD",
                        returnStdout: true
                    ).trim()

                    echo "Changed files:\n${changedFiles}"

                    if (changedFiles.split('\n').any { it == 'Cargo.toml' }) {
                        env.CARGO_CHANGED = "true"
                        echo "✅ Cargo.toml changed"
                    } else {
                        env.CARGO_CHANGED = "false"
                        echo "ℹ️ Cargo.toml not changed"
                    }
                }
            }
        }

        // jika diubah, ubah env.CARGO_CHANGED = true
        // build dahulu pre image
        stage('build pre image') {
            when {
                expression { env.CARGO_CHANGED == "true" }
            }
            steps {
                script {
                    sendTelegramMessage("🧱 Cargo.toml berubah, build pre image dimulai")

                    try {
                        sh """
                        docker buildx build \
                            -f Dockerfile.pre \
                            -t ${IMAGEPRE} .
                        """
                    } catch (Exception e) {
                        def log = currentBuild.rawBuild.getLog(100).join("\n")
                        def message = URLEncoder.encode(
                            "🚨 Error build pre image\n\n${log}",
                            "UTF-8"
                        )
                        sendTelegramMessageError(message)
                        error("❌ Gagal build pre image")
                    }
                }
            }
        }

        stage('build image') {
            steps {
                script {
                    withCredentials([
                        string(credentialsId: 'SST_GITHUB_TOKEN', variable: 'GITHUB_TOKEN')
                    ]){
                    //    sh "docker buildx build --build-arg GITHUB_TOKEN=${GITHUB_TOKEN} -t ${IMAGE}:${TAG} ."
                   
                    try {
                        sh "docker buildx build -f Dockerfile.post --build-arg GITHUB_TOKEN=${GITHUB_TOKEN} -t ${IMAGE} ."
                    } catch (Exception e) {

                        // writeFile file: LOG_FILE, text: "Error build image: ${e.toString()}\n"
                        def log = currentBuild.rawBuild.getLog(100).join("\n")
                        def message = URLEncoder.encode("🚨 Error build image *${e.toString()}*\n\n${log}", "UTF-8")
                        sendTelegramMessageError(message)
                        
                        error("❌ Gagal build Docker image.")
                    }
                   
                    }
                }
            }
        }

        stage('Push Docker Image') {
            steps {
                withCredentials([usernamePassword(credentialsId: 'test-cicd-docker', 
                    usernameVariable: 'DOCKER_USER', passwordVariable: 'DOCKER_PASS')]) {
                    script {
                        try {
                            // Login ke Docker Hub
                            sh 'echo $DOCKER_PASS | docker login -u $DOCKER_USER --password-stdin'
                        
                            // Tag dan push image
                            sh "docker push ${IMAGE}:${TAG}"
                       } catch (Exception e) {
                            // writeFile file: LOG_FILE, text: "Error push image: ${e.toString()}\n"
                            def log = currentBuild.rawBuild.getLog(100).join("\n")
                            def message = URLEncoder.encode("🚨 Error push image: *${e.toString()}*\n\n${log}", "UTF-8")
                            sendTelegramMessageError(message)
                            
                            error("❌ Gagal push Docker image.")
                        }
                    }
                }
            }
        }

        // karena satu pid docker, kita tidak butuh step download image
    }

    post {
        success {
            script {
                sendTelegramMessage("✅ Build sukses! Docker image ${IMAGE}:${TAG} telah di-push.")
            }
            echo "build and push ${IMAGE}:${TAG}"
        }
        failure {
             script {
                sendTelegramMessage("❌ Build atau push gagal ${IMAGE}:${TAG}!")
                sendTelegramMessageError()
                // sendTelegramFile(LOG_FILE)

            }
            echo "Build atau push gagal ${IMAGE}:${TAG}. Log dikirim ke Telegram."
        }
    }
}

def sendTelegramMessage(String message) {
    withCredentials([
        string(credentialsId: 'TELE_BOT_TIRA_TOKEN', variable: 'TELEGRAM_BOT_TOKEN'),
        string(credentialsId: 'TELE_BOT_TIRA_CHAT_ID_PLAYGROUND_BOT', variable: 'TELEGRAM_CHAT_ID')
    ]) {
        
        sh """
            curl -s -X POST "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/sendMessage" \\
            -d chat_id=${TELEGRAM_CHAT_ID} \\
            -d text="${message}"
        """
        
    }
}

def sendTelegramFile(String filePath) {
    withCredentials([
        string(credentialsId: 'TELE_BOT_TIRA_TOKEN', variable: 'TELEGRAM_BOT_TOKEN'),
        string(credentialsId: 'TELE_BOT_TIRA_CHAT_ID_PLAYGROUND_BOT', variable: 'TELEGRAM_CHAT_ID')
    ]) {
        
        sh """
            curl -F chat_id=${TELEGRAM_CHAT_ID} \\
                 -F document=@${filePath} \\
                 "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/sendDocument"
        """
        
    }
}

def sendTelegramMessageError(String message) {
    withCredentials([
        string(credentialsId: 'TELE_BOT_TIRA_TOKEN', variable: 'TELEGRAM_BOT_TOKEN'),
        string(credentialsId: 'TELE_BOT_TIRA_CHAT_ID_PLAYGROUND_BOT', variable: 'TELEGRAM_CHAT_ID')
    ]) {
        
        sh """
            curl -s -X POST https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/sendMessage \
            -d chat_id=${TELEGRAM_CHAT_ID} \
            -d parse_mode=Markdown \
            --data-urlencode text="${message}"
        """
        
    }
}
