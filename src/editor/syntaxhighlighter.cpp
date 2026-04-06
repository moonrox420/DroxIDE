// src/editor/syntaxhighlighter.cpp
#include "syntaxhighlighter.h"

SyntaxHighlighter::SyntaxHighlighter(QTextDocument *document, const QString &language)
    : QSyntaxHighlighter(document)
{
    // Keyword format
    keywordFormat.setForeground(Qt::darkBlue);
    keywordFormat.setFontWeight(QFont::Bold);
    
    // Class format
    classFormat.setForeground(Qt::darkMagenta);
    classFormat.setFontWeight(QFont::Bold);
    
    // Single line comment
    singleLineCommentFormat.setForeground(Qt::darkGreen);
    
    // Multi-line comment
    multiLineCommentFormat.setForeground(Qt::darkGreen);
    
    // Quotation
    quotationFormat.setForeground(Qt::darkRed);
    
    // Function
    functionFormat.setForeground(Qt::blue);
    
    // Rust keywords
    if (language == "rust") {
        QStringList rustKeywords;
        rustKeywords << "\\bfn\\b" << "\\blet\\b" << "\\bmut\\b" << "\\bif\\b" 
                     << "\\belse\\b" << "\\bfor\\b" << "\\bwhile\\b" << "\\bloop\\b"
                     << "\\bstruct\\b" << "\\benum\\b" << "\\bimpl\\b" << "\\btrait\\b"
                     << "\\bpub\\b" << "\\bprivate\\b" << "\\bself\\b" << "\\buse\\b"
                     << "\\bmod\\b" << "\\basync\\b" << "\\bawait\\b" << "\\breturn\\b";
        
        for (const QString &pattern : rustKeywords) {
            HighlightingRule rule;
            rule.pattern = QRegularExpression(pattern);
            rule.format = keywordFormat;
            highlightingRules.append(rule);
        }
    }
    
    // Python keywords
    else if (language == "python") {
        QStringList pythonKeywords;
        pythonKeywords << "\\bdef\\b" << "\\bclass\\b" << "\\bif\\b" << "\\belse\\b"
                       << "\\belif\\b" << "\\bfor\\b" << "\\bwhile\\b" << "\\breturn\\b"
                       << "\\bimport\\b" << "\\bfrom\\b" << "\\bas\\b" << "\\btry\\b"
                       << "\\bexcept\\b" << "\\bfinally\\b" << "\\bwith\\b" << "\\bassert\\b"
                       << "\\bpass\\b" << "\\bbreak\\b" << "\\bcontinue\\b" << "\\byield\\b";
        
        for (const QString &pattern : pythonKeywords) {
            HighlightingRule rule;
            rule.pattern = QRegularExpression(pattern);
            rule.format = keywordFormat;
            highlightingRules.append(rule);
        }
    }
    
    // C++ keywords
    else if (language == "cpp") {
        QStringList cppKeywords;
        cppKeywords << "\\bvoid\\b" << "\\bint\\b" << "\\bfloat\\b" << "\\bdouble\\b"
                    << "\\bchar\\b" << "\\bconst\\b" << "\\bstatic\\b" << "\\bclass\\b"
                    << "\\bstruct\\b" << "\\bif\\b" << "\\belse\\b" << "\\bfor\\b"
                    << "\\bwhile\\b" << "\\breturn\\b" << "\\bnew\\b" << "\\bdelete\\b"
                    << "\\btemplate\\b" << "\\btypename\\b" << "\\bnamespace\\b" << "\\bpublic\\b"
                    << "\\bprivate\\b" << "\\bprotected\\b";
        
        for (const QString &pattern : cppKeywords) {
            HighlightingRule rule;
            rule.pattern = QRegularExpression(pattern);
            rule.format = keywordFormat;
            highlightingRules.append(rule);
        }
    }
    
    // JavaScript/TypeScript keywords
    else if (language == "javascript") {
        QStringList jsKeywords;
        jsKeywords << "\\bfunction\\b" << "\\bvar\\b" << "\\blet\\b" << "\\bconst\\b"
                   << "\\bif\\b" << "\\belse\\b" << "\\bfor\\b" << "\\bwhile\\b"
                   << "\\breturn\\b" << "\\bclass\\b" << "\\bimport\\b" << "\\bexport\\b"
                   << "\\basync\\b" << "\\bawait\\b" << "\\btry\\b" << "\\bcatch\\b";
        
        for (const QString &pattern : jsKeywords) {
            HighlightingRule rule;
            rule.pattern = QRegularExpression(pattern);
            rule.format = keywordFormat;
            highlightingRules.append(rule);
        }
    }
    
    // Numbers
    HighlightingRule numberRule;
    numberRule.pattern = QRegularExpression("\\b[0-9]+\\b");
    numberRule.format.setForeground(Qt::darkCyan);
    highlightingRules.append(numberRule);
    
    // Strings
    HighlightingRule stringRule;
    stringRule.pattern = QRegularExpression("\".*\"");
    stringRule.format = quotationFormat;
    highlightingRules.append(stringRule);
    
    // Comments
    commentStartExpression = QRegularExpression("/\\*");
    commentEndExpression = QRegularExpression("\\*/");
}

void SyntaxHighlighter::highlightBlock(const QString &text)
{
    // Apply highlighting rules
    for (const HighlightingRule &rule : highlightingRules) {
        QRegularExpressionMatchIterator matchIterator = rule.pattern.globalMatch(text);
        while (matchIterator.hasNext()) {
            QRegularExpressionMatch match = matchIterator.next();
            setFormat(match.capturedStart(), match.capturedLength(), rule.format);
        }
    }
    
    // Single line comments
    QRegularExpression singleLineCommentRegex("//.*");
    QRegularExpressionMatchIterator matchIterator = singleLineCommentRegex.globalMatch(text);
    while (matchIterator.hasNext()) {
        QRegularExpressionMatch match = matchIterator.next();
        setFormat(match.capturedStart(), match.capturedLength(), singleLineCommentFormat);
    }
    
    // Multi-line comments
    setCurrentBlockState(0);
    
    int startIndex = 0;
    if (previousBlockState() != 1) {
        startIndex = text.indexOf(commentStartExpression);
    }
    
    while (startIndex >= 0) {
        QRegularExpressionMatch endMatch = commentEndExpression.match(text, startIndex);
        int endIndex = endMatch.capturedStart();
        int commentLength;
        
        if (endIndex == -1) {
            setCurrentBlockState(1);
            commentLength = text.length() - startIndex;
        } else {
            commentLength = endIndex - startIndex + endMatch.capturedLength();
        }
        
        setFormat(startIndex, commentLength, multiLineCommentFormat);
        startIndex = text.indexOf(commentStartExpression, startIndex + commentLength);
    }
}
